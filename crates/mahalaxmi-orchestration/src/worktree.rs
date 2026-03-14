// SPDX-License-Identifier: MIT
// Copyright 2026 ThriveTech Services LLC
//! Git worktree manager — per-worker file isolation.
//!
//! Each worker gets its own git worktree so concurrent workers never conflict
//! on files. Worktrees are created under `.mahalaxmi/worktrees/` in the project
//! root and merged back on worker completion.
//!
//! Uses `git` CLI via `std::process::Command` (no `git2` crate dependency).

use chrono::{DateTime, Utc};
use mahalaxmi_core::config::DirtyBranchPolicy;
use mahalaxmi_core::error::MahalaxmiError;
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::types::{TaskId, WorkerId};
use mahalaxmi_core::MahalaxmiResult;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing;

/// Information about an active worktree.
#[derive(Debug, Clone)]
pub struct WorktreeInfo {
    /// The worker this worktree belongs to.
    pub worker_id: WorkerId,
    /// Absolute path to the worktree directory.
    pub path: PathBuf,
    /// Branch name created for this worktree.
    pub branch_name: String,
    /// When the worktree was created.
    pub created_at: DateTime<Utc>,
}

/// Result of merging a worktree back into the main working tree.
#[derive(Debug, Clone)]
pub enum MergeResult {
    /// Merge completed cleanly.
    Clean,
    /// Merge had conflicts.
    Conflict {
        /// Files with merge conflicts.
        conflicting_files: Vec<String>,
        /// Raw diff summary.
        diff: String,
    },
}

/// Summary of partial progress in a worktree (for failed workers).
#[derive(Debug, Clone)]
pub struct PartialProgress {
    /// Files modified in the worktree relative to branch point.
    pub files_modified: Vec<String>,
    /// Human-readable diff summary.
    pub diff_summary: String,
    /// Number of commits made in the worktree.
    pub commit_count: u32,
}

/// Manages git worktrees for per-worker file isolation.
///
/// Each worker gets a dedicated worktree under `.mahalaxmi/worktrees/`.
/// On success the worktree is merged back; on failure it is discarded
/// (or partially merged if `accept_partial_progress` is enabled).
pub struct WorktreeManager {
    project_root: PathBuf,
    active_worktrees: HashMap<WorkerId, WorktreeInfo>,
    i18n: I18nService,
    /// GitHub/GitLab personal access token injected as `GH_TOKEN` when calling `gh`.
    /// When `None`, `gh` falls back to its own stored credentials.
    github_token: Option<String>,
    /// Cycle label for branch prefix — set from the consensus result after
    /// the manager phase completes. When `Some("security-hardening")`, branches
    /// are named `mahalaxmi/security-hardening-{short_id}/worker-0-task-name`.
    /// When `None`, the legacy format `mahalaxmi/worker-0-task-name` is used.
    cycle_label: Option<String>,
    /// First 8 hex chars of the cycle UUID, appended to `cycle_label` in the
    /// branch path so that re-running the same requirements file never produces
    /// colliding branch names. Set alongside `cycle_label` from `driver.rs`.
    cycle_short_id: Option<String>,
    /// Whether to emit info-level audit logs for git/GitHub operations.
    /// Controlled by `config.logging.git`. Defaults to `true`.
    git_log: bool,
    /// Policy for handling uncommitted changes at cycle start.
    dirty_policy: DirtyBranchPolicy,
    /// Stash label created by `apply_dirty_policy()` when policy is `Stash`.
    /// `restore_pre_cycle_stash()` pops this stash at cycle end.
    pre_cycle_stash: Option<String>,
}

impl WorktreeManager {
    /// Create a new worktree manager for the given project root.
    ///
    /// Verifies that `project_root` is a git repository and that `git` is
    /// available on PATH. Then applies `dirty_policy`:
    /// - `Abort` → returns `Err` listing uncommitted files (no state changed).
    /// - `Stash` → auto-stashes uncommitted changes; call `restore_pre_cycle_stash()` at cycle end.
    /// - `Ignore` → proceeds with a `warn!` log.
    pub fn new(
        project_root: PathBuf,
        i18n: I18nService,
        dirty_policy: DirtyBranchPolicy,
    ) -> MahalaxmiResult<Self> {
        // Verify git is available
        let git_check = Command::new("git").arg("--version").output().map_err(|e| {
            MahalaxmiError::platform(
                &i18n,
                "worktree-git-not-found",
                &[("detail", &e.to_string())],
            )
        })?;
        if !git_check.status.success() {
            return Err(MahalaxmiError::platform(
                &i18n,
                "worktree-git-check-failed",
                &[],
            ));
        }

        // Verify project_root is a git repo
        let repo_check = run_git(
            &project_root,
            &["rev-parse", "--is-inside-work-tree"],
            &i18n,
        )?;
        if repo_check.trim() != "true" {
            return Err(MahalaxmiError::platform(
                &i18n,
                "worktree-not-git-repo",
                &[("path", &project_root.display().to_string())],
            ));
        }

        let mut mgr = Self {
            project_root,
            active_worktrees: HashMap::new(),
            i18n,
            github_token: None,
            cycle_label: None,
            cycle_short_id: None,
            git_log: true,
            dirty_policy,
            pre_cycle_stash: None,
        };

        mgr.apply_dirty_policy()?;

        Ok(mgr)
    }

    /// Check for uncommitted changes in the working tree.
    ///
    /// Returns a list of status lines from `git status --porcelain`.
    /// An empty vec means the working tree is clean.
    fn check_dirty_state(&self) -> MahalaxmiResult<Vec<String>> {
        let output = run_git(&self.project_root, &["status", "--porcelain"], &self.i18n)?;
        Ok(output
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(String::from)
            .collect())
    }

    /// Apply the configured `dirty_policy` to the working tree.
    ///
    /// Called once during `new()`. Errors from `Abort` cause `new()` to fail;
    /// `Stash` records the stash label in `pre_cycle_stash` for later restore.
    fn apply_dirty_policy(&mut self) -> MahalaxmiResult<()> {
        let dirty = self.check_dirty_state()?;
        if dirty.is_empty() {
            return Ok(());
        }
        match self.dirty_policy {
            DirtyBranchPolicy::Abort => Err(MahalaxmiError::Platform {
                message: format!(
                    "Working tree has uncommitted changes — commit or stash before starting a cycle.\n  {}",
                    dirty.join("\n  ")
                ),
                i18n_key: "git.dirty_abort_message".to_owned(),
            }),
            DirtyBranchPolicy::Stash => {
                let label = format!(
                    "mahalaxmi-pre-cycle-{}",
                    chrono::Utc::now().timestamp()
                );
                run_git(
                    &self.project_root,
                    &["stash", "push", "-m", &label],
                    &self.i18n,
                )?;
                self.pre_cycle_stash = Some(label.clone());
                tracing::info!(policy = "stash", stash_label = %label, "git: stashed dirty working tree before cycle");
                Ok(())
            }
            DirtyBranchPolicy::Ignore => {
                tracing::warn!(
                    files = ?dirty,
                    "git: starting cycle with dirty working tree (policy=ignore)"
                );
                Ok(())
            }
        }
    }

    /// Restore the pre-cycle stash created by `apply_dirty_policy()`.
    ///
    /// Call this at cycle end (both success and failure paths). A no-op when
    /// the policy was not `Stash` or the stash was never created. Failures are
    /// logged at `warn` level — the stash is preserved so the user can recover
    /// manually via `git stash pop`.
    pub fn restore_pre_cycle_stash(&self) -> MahalaxmiResult<()> {
        if self.pre_cycle_stash.is_some() {
            if let Err(e) = run_git(&self.project_root, &["stash", "pop"], &self.i18n) {
                tracing::warn!(
                    error = %e,
                    "git: failed to restore pre-cycle stash — stash is preserved, run `git stash pop` manually"
                );
            } else {
                tracing::info!("git: restored pre-cycle stash");
            }
        }
        Ok(())
    }

    /// Set the cycle label used as a git branch path prefix.
    ///
    /// Must be called before `create_worktree()` to take effect. The label
    /// is produced by the consensus engine from manager JSON output.
    /// Combined with the short cycle ID (see `set_cycle_short_id`), branches
    /// are named `mahalaxmi/security-hardening-cd73aa79/worker-0-task-name`.
    pub fn set_cycle_label(&mut self, label: String) {
        self.cycle_label = Some(label);
    }

    /// Set the short cycle ID (first 8 hex chars of the cycle UUID) appended
    /// to the cycle label in branch names.
    ///
    /// This guarantees branch name uniqueness across repeated runs of the same
    /// requirements file — identical cycle labels never produce colliding names.
    /// Call immediately after `set_cycle_label`.
    pub fn set_cycle_short_id(&mut self, short_id: String) {
        self.cycle_short_id = Some(short_id);
    }

    /// Set the GitHub/GitLab personal access token used for `gh`/`glab` commands.
    ///
    /// This token is injected as the `GH_TOKEN` environment variable when
    /// spawning `gh pr create` and `gh pr merge` subprocesses. Required when
    /// the Tauri process does not inherit the correct token from the shell
    /// environment (e.g., when launched from a desktop shortcut on Linux/macOS).
    pub fn set_github_token(&mut self, token: String) {
        self.github_token = Some(token);
    }

    /// Enable or disable info-level audit logging for git/GitHub operations.
    ///
    /// Controlled by `config.logging.git`. When `false`, lifecycle info logs
    /// (worktree created, branch pushed, PR URL, merge outcome) are suppressed.
    /// `warn!` and `error!` logs are always emitted regardless of this flag.
    pub fn set_git_log(&mut self, enabled: bool) {
        self.git_log = enabled;
    }

    /// Create a worktree for the given worker.
    ///
    /// Creates `.mahalaxmi/worktrees/worker-{id}/` with a dedicated branch.
    /// Ensures `.mahalaxmi/` is in `.gitignore`.
    pub fn create_worktree(
        &mut self,
        worker_id: WorkerId,
        task_id: &TaskId,
    ) -> MahalaxmiResult<WorktreeInfo> {
        // If worktree already exists for this worker, return it
        if let Some(existing) = self.active_worktrees.get(&worker_id) {
            return Ok(existing.clone());
        }

        if self.git_log {
            tracing::info!(worker_id = %worker_id, task = %task_id.as_str(), "git: creating worktree");
        }

        // Ensure .mahalaxmi/ is in .gitignore
        self.ensure_gitignore()?;

        let worktree_dir = self
            .project_root
            .join(".mahalaxmi")
            .join("worktrees")
            .join(format!("worker-{}", worker_id.as_u32()));

        let branch_name = match (&self.cycle_label, &self.cycle_short_id) {
            (Some(label), Some(short_id)) => format!(
                "mahalaxmi/{}-{}/worker-{}-{}",
                label,
                short_id,
                worker_id.as_u32(),
                sanitize_branch_name(task_id.as_str()),
            ),
            (Some(label), None) => format!(
                "mahalaxmi/{}/worker-{}-{}",
                label,
                worker_id.as_u32(),
                sanitize_branch_name(task_id.as_str()),
            ),
            (None, _) => format!(
                "mahalaxmi/worker-{}-{}",
                worker_id.as_u32(),
                sanitize_branch_name(task_id.as_str()),
            ),
        };

        // Create parent directory
        if let Some(parent) = worktree_dir.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                MahalaxmiError::platform(
                    &self.i18n,
                    "worktree-dir-create-failed",
                    &[
                        ("path", &parent.display().to_string()),
                        ("detail", &e.to_string()),
                    ],
                )
            })?;
        }

        // Remove stale branch from a previous cycle if it exists.
        // `git worktree add -b` fails when the branch already exists,
        // so we must clean it up first. "Branch not found" is the common case
        // on the first cycle or when task names change — log at DEBUG, not WARN.
        if let Err(e) = run_git_silent(
            &self.project_root,
            &["branch", "-D", &branch_name],
            &self.i18n,
        ) {
            tracing::debug!(
                branch = %branch_name,
                error = %e,
                "stale branch cleanup: branch not found (expected for new task)"
            );
        }

        // Prune any stale worktree references that point to
        // directories already removed by a previous cycle.
        let _ = run_git_silent(&self.project_root, &["worktree", "prune"], &self.i18n);

        // Remove the worktree directory from disk if it still exists from a
        // previous cycle. `git worktree prune` only cleans git's internal
        // metadata — the directory itself persists and causes `git worktree add`
        // to fail with "already exists" on the next cycle run.
        if worktree_dir.exists() {
            let _ = std::fs::remove_dir_all(&worktree_dir);
        }

        // Create worktree with a new branch.
        // If the path is "missing but already registered" from a prior aborted
        // cycle (git worktree prune does not always clear this immediately),
        // retry with --force per git's own suggestion. We also delete any
        // branch that the first attempt may have partially created before it
        // failed, so the --force retry can create it fresh.
        let add_result = run_git(
            &self.project_root,
            &[
                "worktree",
                "add",
                worktree_dir.to_str().unwrap_or(""),
                "-b",
                &branch_name,
            ],
            &self.i18n,
        );
        if let Err(e) = add_result {
            let msg = e.to_string();
            if msg.contains("already registered worktree") || msg.contains("missing but already") {
                tracing::warn!(
                    worker_id = %worker_id,
                    path = %worktree_dir.display(),
                    "git: stale worktree registration — retrying with --force"
                );
                // The first attempt may have created the branch before failing.
                // Delete it so the retry can recreate it from HEAD cleanly.
                // "Branch not found" is fine here — log at DEBUG, not WARN.
                if let Err(e) = run_git_silent(
                    &self.project_root,
                    &["branch", "-D", &branch_name],
                    &self.i18n,
                ) {
                    tracing::debug!(
                        branch = %branch_name,
                        error = %e,
                        "stale branch cleanup on retry: branch not found"
                    );
                }
                run_git(
                    &self.project_root,
                    &[
                        "worktree",
                        "add",
                        "--force",
                        worktree_dir.to_str().unwrap_or(""),
                        "-b",
                        &branch_name,
                    ],
                    &self.i18n,
                )?;
            } else {
                return Err(e);
            }
        }

        let info = WorktreeInfo {
            worker_id,
            path: worktree_dir,
            branch_name,
            created_at: Utc::now(),
        };

        self.active_worktrees.insert(worker_id, info.clone());
        if self.git_log {
            tracing::info!(
                worker_id = %worker_id,
                branch = %info.branch_name,
                path = %info.path.display(),
                "git: worktree ready"
            );
        }
        Ok(info)
    }

    /// Remove a worktree and its branch.
    pub fn remove_worktree(&mut self, worker_id: WorkerId) -> MahalaxmiResult<()> {
        let info = match self.active_worktrees.remove(&worker_id) {
            Some(info) => info,
            None => return Ok(()), // Already removed or never created
        };

        if self.git_log {
            tracing::info!(worker_id = %worker_id, branch = %info.branch_name, "git: removing worktree");
        }

        // Remove the worktree (force in case of uncommitted changes).
        // Failure is expected when the directory was already removed by a prior
        // cleanup or when the path is stale — log at DEBUG, not WARN.
        if let Err(e) = run_git_silent(
            &self.project_root,
            &[
                "worktree",
                "remove",
                "--force",
                info.path.to_str().unwrap_or(""),
            ],
            &self.i18n,
        ) {
            tracing::debug!(path = %info.path.display(), error = %e, "worktree remove: path already gone (expected)");
        }

        // Delete the branch. Failure is expected when GitHub already deleted the
        // branch on PR auto-merge — log at DEBUG, not WARN.
        if let Err(e) = run_git_silent(
            &self.project_root,
            &["branch", "-D", &info.branch_name],
            &self.i18n,
        ) {
            tracing::debug!(branch = %info.branch_name, error = %e, "branch delete: not found (expected after auto-merge)");
        }

        Ok(())
    }

    /// Merge a worktree's branch back into the current branch.
    ///
    /// Returns `MergeResult::Clean` on success, or `MergeResult::Conflict`
    /// with details on failure. Does NOT remove the worktree — caller decides.
    pub fn merge_worktree(&self, worker_id: WorkerId) -> MahalaxmiResult<MergeResult> {
        let info = self
            .active_worktrees
            .get(&worker_id)
            .ok_or_else(|| MahalaxmiError::orchestration(&self.i18n, "worktree-not-found", &[]))?;

        // Check if there are any commits to merge
        let log_output = run_git(
            &self.project_root,
            &["log", "--oneline", &format!("HEAD..{}", info.branch_name)],
            &self.i18n,
        )?;

        if log_output.trim().is_empty() {
            // No commits in worktree — nothing to merge
            return Ok(MergeResult::Clean);
        }

        if self.git_log {
            tracing::info!(worker_id = %worker_id, branch = %info.branch_name, "git: merging worker branch");
        }

        // Attempt merge
        let merge_result = Command::new("git")
            .args([
                "merge",
                "--no-ff",
                "-m",
                &format!(
                    "mahalaxmi: merge worker-{} ({})",
                    worker_id.as_u32(),
                    info.branch_name
                ),
                &info.branch_name,
            ])
            .current_dir(&self.project_root)
            .output()
            .map_err(|e| {
                MahalaxmiError::platform(
                    &self.i18n,
                    "worktree-merge-exec-failed",
                    &[("detail", &e.to_string())],
                )
            })?;

        if merge_result.status.success() {
            if self.git_log {
                tracing::info!(worker_id = %worker_id, branch = %info.branch_name, "git: worker branch merged");
            }
            Ok(MergeResult::Clean)
        } else {
            // Merge failed — collect conflict info
            let conflict_output = run_git(
                &self.project_root,
                &["diff", "--name-only", "--diff-filter=U"],
                &self.i18n,
            )
            .unwrap_or_default();

            let conflicting_files: Vec<String> = conflict_output
                .lines()
                .filter(|l| !l.is_empty())
                .map(String::from)
                .collect();

            let diff = String::from_utf8_lossy(&merge_result.stderr).to_string();

            // Abort the failed merge
            let _ = run_git(&self.project_root, &["merge", "--abort"], &self.i18n);

            Ok(MergeResult::Conflict {
                conflicting_files,
                diff,
            })
        }
    }

    /// Extract partial progress from a worktree (for failed workers).
    ///
    /// Returns the list of files modified and a diff summary.
    pub fn extract_partial_progress(
        &self,
        worker_id: WorkerId,
    ) -> MahalaxmiResult<PartialProgress> {
        let info = self
            .active_worktrees
            .get(&worker_id)
            .ok_or_else(|| MahalaxmiError::orchestration(&self.i18n, "worktree-not-found", &[]))?;

        // Get files modified in the worktree branch
        let files_output = run_git(
            &self.project_root,
            &[
                "diff",
                "--name-only",
                &format!("HEAD...{}", info.branch_name),
            ],
            &self.i18n,
        )
        .unwrap_or_default();

        let files_modified: Vec<String> = files_output
            .lines()
            .filter(|l| !l.is_empty())
            .map(String::from)
            .collect();

        // Get diff stat summary
        let diff_summary = run_git(
            &self.project_root,
            &["diff", "--stat", &format!("HEAD...{}", info.branch_name)],
            &self.i18n,
        )
        .unwrap_or_default();

        // Count commits
        let log_output = run_git(
            &self.project_root,
            &[
                "rev-list",
                "--count",
                &format!("HEAD..{}", info.branch_name),
            ],
            &self.i18n,
        )
        .unwrap_or_default();
        let commit_count = log_output.trim().parse::<u32>().unwrap_or(0);

        Ok(PartialProgress {
            files_modified,
            diff_summary,
            commit_count,
        })
    }

    /// Remove all worktrees and prune.
    pub fn cleanup_all(&mut self) -> MahalaxmiResult<()> {
        let worker_ids: Vec<WorkerId> = self.active_worktrees.keys().copied().collect();
        for worker_id in worker_ids {
            self.remove_worktree(worker_id)?;
        }

        // Prune any stale worktree references
        let _ = run_git_silent(&self.project_root, &["worktree", "prune"], &self.i18n);

        // Remove the worktrees directory if empty
        let worktrees_dir = self.project_root.join(".mahalaxmi").join("worktrees");
        if worktrees_dir.exists() {
            let _ = std::fs::remove_dir(&worktrees_dir);
        }

        Ok(())
    }

    /// Get info about an active worktree.
    pub fn get_worktree(&self, worker_id: WorkerId) -> Option<&WorktreeInfo> {
        self.active_worktrees.get(&worker_id)
    }

    /// List all active worktrees.
    pub fn list_active(&self) -> Vec<&WorktreeInfo> {
        self.active_worktrees.values().collect()
    }

    /// Checkout a specific branch in the project root, pulling the latest
    /// remote state first so that all worker worktrees start from the same
    /// up-to-date base.
    ///
    /// A fetch failure is logged but not fatal — workers will still start
    /// from whatever the local branch currently points at.
    pub fn checkout_target_branch(&self, branch: &str) -> MahalaxmiResult<()> {
        // Pull latest from origin so all workers branch from the same
        // up-to-date point, reducing the chance of mid-cycle conflicts.
        if let Err(e) = run_git(&self.project_root, &["fetch", "origin", branch], &self.i18n) {
            tracing::warn!(
                branch = %branch,
                error = %e,
                "fetch before checkout failed — proceeding with local branch state"
            );
        }
        run_git(&self.project_root, &["checkout", branch], &self.i18n)?;
        // Stash any uncommitted local changes so they don't block the
        // fast-forward.  This is the common case when a previous cycle left
        // generated files (e.g. Cargo.lock, a routes file) modified in the
        // main checkout.  After a successful fast-forward we pop the stash;
        // if the stash conflicts with the incoming commits (the file is
        // already present in origin) we drop it to leave the tree clean.
        if self.git_log {
            tracing::info!(branch = %branch, "git: target branch checked out");
        }
        let stash_out = run_git(&self.project_root, &["stash"], &self.i18n).unwrap_or_default();
        let stashed = !stash_out.contains("No local changes to save");
        // Fast-forward local branch to match remote.
        if let Err(e) = run_git(
            &self.project_root,
            &["merge", "--ff-only", &format!("origin/{}", branch)],
            &self.i18n,
        ) {
            tracing::warn!(
                branch = %branch,
                error = %e,
                "fast-forward after fetch failed — workers will branch from local state"
            );
            // Restore stashed changes so local work is not lost.
            if stashed {
                if let Err(e) = run_git_silent(&self.project_root, &["stash", "pop"], &self.i18n) {
                    tracing::debug!(error = %e, "stash pop after failed fast-forward: nothing to restore");
                }
            }
        } else if stashed {
            // Fast-forward succeeded; restore stashed local work.  If the
            // pop conflicts (the incoming commits already contain the same
            // change), drop the stash — the canonical version is now in HEAD.
            if run_git(&self.project_root, &["stash", "pop"], &self.i18n).is_err() {
                if let Err(e) = run_git_silent(&self.project_root, &["stash", "drop"], &self.i18n) {
                    tracing::debug!(error = %e, "stash drop: nothing to drop");
                }
            }
        }
        Ok(())
    }

    /// Push a worker branch to the remote.
    /// Merge the target branch into the worker branch, then push.
    ///
    /// When `target_branch` is `Some(branch)`, fetches `origin/<branch>` and
    /// first attempts a no-fast-forward merge with `-X ours` so file-level
    /// conflicts are resolved by keeping the worker's own changes (each
    /// worker's task scope is authoritative for its assigned files — C1).
    /// If the merge fails for a non-conflict reason, it is aborted and a
    /// `git rebase -X theirs origin/<branch>` is attempted as a fallback
    /// (in rebase context "theirs" = the worker's patches, so the same
    /// worker-wins policy applies).  Rebase rewrites commit hashes but is
    /// safe because `--force-with-lease` is used on the push; the lease is
    /// refreshed by fetching the worker's own remote branch immediately before
    /// the push.
    ///
    /// Only if the rebase also fails (e.g. binary-file or delete/modify
    /// conflicts that `-X theirs` cannot resolve) is the error surfaced to
    /// the caller as a `WorkerConflict` event. Warnings are logged at each
    /// stage.
    ///
    /// Used by the `BranchAndPr` strategy.
    pub fn push_branch(
        &self,
        worker_id: WorkerId,
        target_branch: Option<&str>,
    ) -> MahalaxmiResult<()> {
        let info = self
            .active_worktrees
            .get(&worker_id)
            .ok_or_else(|| MahalaxmiError::orchestration(&self.i18n, "worktree-not-found", &[]))?;

        if let Some(target) = target_branch {
            // Commit any pending (uncommitted) changes in the worktree before
            // merging remote changes.  The most common source is Cargo.lock
            // regenerated by `cargo build`/`cargo check` in the verify step,
            // which runs after the worker's last explicit commit.  Without
            // this step, git refuses to merge with "your local changes to
            // Cargo.lock would be overwritten".  Errors are intentionally
            // ignored: if there is nothing to stage the add is a no-op, and
            // the commit fails silently with "nothing to commit".
            if let Err(e) = run_git_silent(&info.path, &["add", "-A"], &self.i18n) {
                tracing::debug!(error = %e, "pre-merge add -A: nothing to stage");
            }
            if let Err(e) = run_git_silent(
                &info.path,
                &["commit", "-m", "chore: generated files (Cargo.lock etc.)"],
                &self.i18n,
            ) {
                tracing::debug!(error = %e, "pre-merge commit: nothing to commit");
            }

            run_git(&info.path, &["fetch", "origin", target], &self.i18n)?;
            let remote_ref = format!("origin/{}", target);

            // Attempt a no-fast-forward merge first.  Use -X ours so that
            // any file-level conflicts are resolved by keeping the worker's
            // own changes.  Each worker's task scope is authoritative for
            // its assigned files (C1 constraint); a concurrent worker that
            // modified the same file has violated exclusive-scope and its
            // stale version must not overwrite this worker's result.
            let merge_result = run_git(
                &info.path,
                &["merge", "--no-ff", "--no-edit", "-X", "ours", &remote_ref],
                &self.i18n,
            );

            if let Err(merge_err) = merge_result {
                // Merge failed (typically a concurrent worker merged conflicting
                // changes while this worker was running).  Abort the failed merge
                // to restore a clean worktree, then try rebase as a fallback.
                tracing::warn!(
                    worker_id = %worker_id,
                    target = %remote_ref,
                    error = %merge_err,
                    "merge failed; aborting and attempting rebase fallback"
                );
                let _ = run_git(&info.path, &["merge", "--abort"], &self.i18n);

                // Rebase rewrites commit hashes — safe because the push below
                // uses --force-with-lease with a fresh fetch to guard the lease.
                // -X theirs: in a rebase "theirs" is the patch being applied
                // (the worker's commit), so conflicts resolve in the worker's
                // favour — consistent with the -X ours preference used above.
                if let Err(rebase_err) = run_git(
                    &info.path,
                    &["rebase", "-X", "theirs", &remote_ref],
                    &self.i18n,
                ) {
                    // Both merge and rebase failed: genuine overlapping edits.
                    // Abort the rebase to leave the worktree clean and surface
                    // the original merge error to the caller.
                    tracing::warn!(
                        worker_id = %worker_id,
                        target = %remote_ref,
                        error = %rebase_err,
                        "rebase fallback also failed; aborting — emitting conflict"
                    );
                    let _ = run_git(&info.path, &["rebase", "--abort"], &self.i18n);
                    return Err(merge_err);
                }

                tracing::info!(
                    worker_id = %worker_id,
                    target = %remote_ref,
                    "rebase fallback succeeded after merge conflict"
                );
            }
        }

        // Fetch the worker's own branch before pushing so that --force-with-lease
        // has an accurate lease expectation.  Without this fetch, the lease
        // defaults to "null" (branch should not exist), which causes the push to
        // be rejected whenever the same branch name was used in a prior cycle.
        // "Branch not found" is expected for every new branch — log at DEBUG.
        if let Err(e) = run_git_silent(
            &info.path,
            &["fetch", "origin", &info.branch_name],
            &self.i18n,
        ) {
            tracing::debug!(
                branch = %info.branch_name,
                error = %e,
                "pre-push fetch: branch not yet on remote (expected for new branch)"
            );
        }

        // Use --force-with-lease: safe force-push that rejects if the remote
        // has changed since our fetch, protecting against concurrent pushes.
        run_git(
            &self.project_root,
            &[
                "push",
                "--force-with-lease",
                "-u",
                "origin",
                &info.branch_name,
            ],
            &self.i18n,
        )?;
        if self.git_log {
            tracing::info!(
                worker_id = %worker_id,
                branch = %info.branch_name,
                "git: worker branch pushed to remote"
            );
        }
        Ok(())
    }

    /// Returns `true` if the worktree for `worker_id` has at least one commit
    /// ahead of `origin/{base_branch}`.
    ///
    /// Returns `true` on any git error so that the push path is not skipped
    /// incorrectly. Returns `false` only when the count is definitively zero
    /// (the task was already complete in the codebase before the worker ran).
    pub fn worker_has_new_commits(&self, worker_id: WorkerId, base_branch: &str) -> bool {
        let Some(info) = self.active_worktrees.get(&worker_id) else {
            return false;
        };
        let remote_ref = format!("origin/{}", base_branch);
        let Ok(out) = run_git(
            &info.path,
            &["rev-list", "--count", &format!("{}..HEAD", remote_ref)],
            &self.i18n,
        ) else {
            // On error assume commits exist — do not skip the push.
            return true;
        };
        out.trim().parse::<u32>().unwrap_or(0) > 0
    }

    /// Create a pull request / merge request for a worker branch.
    ///
    /// Uses `gh pr create` (GitHub) or `glab mr create` (GitLab).
    /// Returns the PR/MR URL on success.
    pub fn create_pull_request(
        &self,
        worker_id: WorkerId,
        target_branch: &str,
        platform: mahalaxmi_core::types::GitPrPlatform,
        auto_merge: bool,
    ) -> MahalaxmiResult<String> {
        let info = self
            .active_worktrees
            .get(&worker_id)
            .ok_or_else(|| MahalaxmiError::orchestration(&self.i18n, "worktree-not-found", &[]))?;

        let title = format!(
            "mahalaxmi: worker-{} ({})",
            worker_id.as_u32(),
            info.branch_name
        );

        let platform_name = match platform {
            mahalaxmi_core::types::GitPrPlatform::GitHub => "github",
            mahalaxmi_core::types::GitPrPlatform::GitLab => "gitlab",
        };
        if self.git_log {
            tracing::info!(
                worker_id = %worker_id,
                platform = %platform_name,
                title = %title,
                base = %target_branch,
                "git: creating pull request"
            );
        }

        let pr_url = match platform {
            mahalaxmi_core::types::GitPrPlatform::GitHub => {
                let args = vec![
                    "pr",
                    "create",
                    "--title",
                    &title,
                    "--body",
                    "Automated PR created by Mahalaxmi orchestration.",
                    "--base",
                    target_branch,
                    "--head",
                    &info.branch_name,
                ];
                let token = self.github_token.as_deref();
                let url = run_gh(&self.project_root, &args, token, &self.i18n)?;
                let url = url.trim().to_string();
                if self.git_log {
                    tracing::info!(worker_id = %worker_id, pr_url = %url, "git: pull request created");
                }
                if auto_merge {
                    // Immediately merge the PR. `gh pr create --auto` only enables
                    // GitHub's native auto-merge feature, which requires branch
                    // protection rules — it never fires on repos without them.
                    if self.git_log {
                        tracing::info!(worker_id = %worker_id, pr_url = %url, "git: auto-merging pull request");
                    }
                    run_gh(
                        &self.project_root,
                        &["pr", "merge", &url, "--merge"],
                        token,
                        &self.i18n,
                    )?;
                    if self.git_log {
                        tracing::info!(worker_id = %worker_id, pr_url = %url, "git: pull request merged");
                    }
                }
                url
            }
            mahalaxmi_core::types::GitPrPlatform::GitLab => {
                let args = vec![
                    "mr",
                    "create",
                    "--title",
                    &title,
                    "--description",
                    "Automated MR created by Mahalaxmi orchestration.",
                    "--target-branch",
                    target_branch,
                    "--source-branch",
                    &info.branch_name,
                    "--yes",
                ];
                let token = self.github_token.as_deref();
                let url = run_glab(&self.project_root, &args, token, &self.i18n)?;
                let url = url.trim().to_string();
                if self.git_log {
                    tracing::info!(worker_id = %worker_id, pr_url = %url, "git: pull request created");
                }
                if auto_merge {
                    // `glab mr create --auto-merge` only works when pipelines are
                    // configured. Instead, extract the MR IID from the returned URL
                    // (…/merge_requests/{iid}) and merge immediately.
                    let iid = url.rsplit('/').next().unwrap_or("");
                    if self.git_log {
                        tracing::info!(worker_id = %worker_id, pr_url = %url, "git: auto-merging pull request");
                    }
                    run_glab(
                        &self.project_root,
                        &["mr", "merge", iid, "--yes"],
                        token,
                        &self.i18n,
                    )?;
                    if self.git_log {
                        tracing::info!(worker_id = %worker_id, pr_url = %url, "git: pull request merged");
                    }
                }
                url
            }
        };

        Ok(pr_url)
    }

    /// Remove only the worktree directory but keep the branch (for `BranchOnly` strategy).
    pub fn remove_worktree_keep_branch(&mut self, worker_id: WorkerId) -> MahalaxmiResult<()> {
        let info = match self.active_worktrees.remove(&worker_id) {
            Some(info) => info,
            None => return Ok(()),
        };

        if let Err(e) = run_git_silent(
            &self.project_root,
            &[
                "worktree",
                "remove",
                "--force",
                info.path.to_str().unwrap_or(""),
            ],
            &self.i18n,
        ) {
            tracing::debug!(path = %info.path.display(), error = %e, "worktree remove (keep-branch): path already gone");
        }

        // Intentionally do NOT delete the branch
        Ok(())
    }

    /// Ensure `.mahalaxmi/` is in the project's `.gitignore`.
    fn ensure_gitignore(&self) -> MahalaxmiResult<()> {
        let gitignore_path = self.project_root.join(".gitignore");
        let entry = ".mahalaxmi/";

        if gitignore_path.exists() {
            let content = std::fs::read_to_string(&gitignore_path).map_err(|e| {
                MahalaxmiError::platform(
                    &self.i18n,
                    "worktree-gitignore-read-failed",
                    &[("detail", &e.to_string())],
                )
            })?;

            if content.lines().any(|line| line.trim() == entry) {
                return Ok(()); // Already present
            }

            // Append
            let mut new_content = content;
            if !new_content.ends_with('\n') {
                new_content.push('\n');
            }
            new_content.push_str(entry);
            new_content.push('\n');

            std::fs::write(&gitignore_path, new_content).map_err(|e| {
                MahalaxmiError::platform(
                    &self.i18n,
                    "worktree-gitignore-write-failed",
                    &[("detail", &e.to_string())],
                )
            })?;
        } else {
            // Create new .gitignore
            std::fs::write(&gitignore_path, format!("{entry}\n")).map_err(|e| {
                MahalaxmiError::platform(
                    &self.i18n,
                    "worktree-gitignore-create-failed",
                    &[("detail", &e.to_string())],
                )
            })?;
        }

        Ok(())
    }
}

/// Sanitize a string for use as a git branch name component.
fn sanitize_branch_name(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect::<String>()
        .chars()
        .take(40) // Limit branch name length
        .collect()
}

/// Run a `gh` CLI command and return its stdout as a String.
///
/// When `token` is `Some`, it is injected as the `GH_TOKEN` environment
/// variable so that `gh` authenticates with the correct account regardless
/// of what credentials are stored in `~/.config/gh/`.
fn run_gh(
    cwd: &Path,
    args: &[&str],
    token: Option<&str>,
    i18n: &I18nService,
) -> MahalaxmiResult<String> {
    run_cli_tool("gh", cwd, args, token, i18n)
}

/// Run a `glab` CLI command and return its stdout as a String.
///
/// When `token` is `Some`, it is injected as the `GITLAB_TOKEN` environment
/// variable for `glab` authentication.
fn run_glab(
    cwd: &Path,
    args: &[&str],
    token: Option<&str>,
    i18n: &I18nService,
) -> MahalaxmiResult<String> {
    run_cli_tool("glab", cwd, args, token, i18n)
}

/// Run a CLI tool command and return its stdout as a String.
///
/// `token` is injected as `GH_TOKEN` (for `gh`) or `GITLAB_TOKEN` (for `glab`)
/// when provided. This ensures the correct account is used even when the Tauri
/// process does not inherit the shell's environment variables.
fn run_cli_tool(
    tool: &str,
    cwd: &Path,
    args: &[&str],
    token: Option<&str>,
    i18n: &I18nService,
) -> MahalaxmiResult<String> {
    let cmd_str = format!("{} {}", tool, args.join(" "));
    tracing::debug!(tool = %tool, cmd = %cmd_str, cwd = %cwd.display(), has_token = token.is_some(), "gh: executing");
    let mut cmd = Command::new(tool);
    cmd.args(args).current_dir(cwd);
    if let Some(t) = token {
        // gh reads GH_TOKEN; glab reads GITLAB_TOKEN (also accepts GITHUB_TOKEN for gh compat)
        let env_key = if tool == "gh" {
            "GH_TOKEN"
        } else {
            "GITLAB_TOKEN"
        };
        cmd.env(env_key, t);
    }
    let output = cmd.output().map_err(|e| {
        MahalaxmiError::platform(
            i18n,
            "worktree-git-exec-failed",
            &[("cmd", &cmd_str), ("detail", &e.to_string())],
        )
    })?;

    if output.status.success() {
        tracing::debug!(tool = %tool, cmd = %cmd_str, "gh: succeeded");
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        tracing::warn!(tool = %tool, cmd = %cmd_str, stderr = %stderr.trim(), "gh: command failed");
        Err(MahalaxmiError::platform(
            i18n,
            "worktree-git-cmd-failed",
            &[("cmd", &cmd_str), ("detail", stderr.trim())],
        ))
    }
}

/// Run a git command and return its stdout as a String.
///
/// Logs at WARN on non-zero exit. Use this when failure is unexpected and
/// the caller needs to know about it (e.g. push, merge, worktree add).
fn run_git(cwd: &Path, args: &[&str], i18n: &I18nService) -> MahalaxmiResult<String> {
    let cmd_str = format!("git {}", args.join(" "));
    tracing::debug!(cmd = %cmd_str, cwd = %cwd.display(), "git: executing");
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .map_err(|e| {
            MahalaxmiError::platform(
                i18n,
                "worktree-git-exec-failed",
                &[("cmd", &cmd_str), ("detail", &e.to_string())],
            )
        })?;

    if output.status.success() {
        tracing::debug!(cmd = %cmd_str, "git: succeeded");
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stderr_trimmed = stderr.trim();
        tracing::warn!(cmd = %cmd_str, stderr = %stderr_trimmed, "git: command failed");
        Err(MahalaxmiError::platform(
            i18n,
            "worktree-git-cmd-failed",
            &[("cmd", &cmd_str), ("detail", stderr_trimmed)],
        ))
    }
}

/// Run a best-effort git command where failure is expected and normal.
///
/// Identical to `run_git` but logs at DEBUG instead of WARN on non-zero exit.
/// Use this for clean-up operations (branch deletion, pre-push fetch) where
/// "not found" or "nothing to fetch" is the common case, not an error.
fn run_git_silent(cwd: &Path, args: &[&str], i18n: &I18nService) -> MahalaxmiResult<String> {
    let cmd_str = format!("git {}", args.join(" "));
    tracing::debug!(cmd = %cmd_str, cwd = %cwd.display(), "git: executing");
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .map_err(|e| {
            MahalaxmiError::platform(
                i18n,
                "worktree-git-exec-failed",
                &[("cmd", &cmd_str), ("detail", &e.to_string())],
            )
        })?;

    if output.status.success() {
        tracing::debug!(cmd = %cmd_str, "git: succeeded");
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stderr_trimmed = stderr.trim();
        tracing::debug!(cmd = %cmd_str, stderr = %stderr_trimmed, "git: best-effort command returned non-zero (expected)");
        Err(MahalaxmiError::platform(
            i18n,
            "worktree-git-cmd-failed",
            &[("cmd", &cmd_str), ("detail", stderr_trimmed)],
        ))
    }
}
