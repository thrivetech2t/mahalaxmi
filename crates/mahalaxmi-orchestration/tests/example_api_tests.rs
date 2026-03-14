// SPDX-License-Identifier: MIT
// Copyright 2026 ThriveTech Services LLC
//
// Integration tests that lock down the public API surface used by the
// mahalaxmi-orchestration example programs:
//   examples/orchestration/01-dag-types.rs
//   examples/orchestration/02-worker-queue.rs

use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::i18n::locale::SupportedLocale;
use mahalaxmi_core::types::{TaskId, WorkerId};
use mahalaxmi_orchestration::{
    build_phases, detect_cycles, validate_dag, WorkerQueue,
};
use mahalaxmi_orchestration::models::plan::{ExecutionPhase, ExecutionPlan, WorkerTask};
use mahalaxmi_providers::TaskType;

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn i18n() -> I18nService {
    I18nService::new(SupportedLocale::EnUs)
}

// ---------------------------------------------------------------------------
// TaskId
// ---------------------------------------------------------------------------

#[test]
fn task_id_as_str_matches_input() {
    let id = TaskId::new("task-1");
    assert_eq!(id.as_str(), "task-1");
}

#[test]
fn two_task_ids_with_same_value_are_equal() {
    let a = TaskId::new("a");
    let b = TaskId::new("a");
    assert_eq!(a, b);
}

// ---------------------------------------------------------------------------
// WorkerId
// ---------------------------------------------------------------------------

#[test]
fn worker_id_as_u32_matches_input() {
    let id = WorkerId::new(3);
    assert_eq!(id.as_u32(), 3);
}

#[test]
fn two_worker_ids_with_same_value_are_equal() {
    assert_eq!(WorkerId::new(1), WorkerId::new(1));
}

#[test]
fn worker_ids_with_different_values_are_not_equal() {
    assert_ne!(WorkerId::new(1), WorkerId::new(2));
}

// ---------------------------------------------------------------------------
// WorkerTask construction
// ---------------------------------------------------------------------------

#[test]
fn worker_task_new_creates_valid_task() {
    let task = WorkerTask::new(
        TaskId::new("task-1"),
        WorkerId::new(1),
        "title",
        "desc",
    );
    assert_eq!(task.task_id.as_str(), "task-1");
}

#[test]
fn worker_task_task_id_matches() {
    let task = WorkerTask::new(TaskId::new("task-1"), WorkerId::new(1), "t", "d");
    assert_eq!(task.task_id.as_str(), "task-1");
}

#[test]
fn worker_task_title_matches() {
    let task = WorkerTask::new(TaskId::new("t1"), WorkerId::new(0), "title", "desc");
    assert_eq!(task.title, "title");
}

#[test]
fn worker_task_description_matches() {
    let task = WorkerTask::new(TaskId::new("t1"), WorkerId::new(0), "title", "desc");
    assert_eq!(task.description, "desc");
}

#[test]
fn worker_task_dependencies_empty_by_default() {
    let task = WorkerTask::new(TaskId::new("t1"), WorkerId::new(0), "t", "d");
    assert!(task.dependencies.is_empty());
}

#[test]
fn worker_task_set_dependencies_works() {
    let dep = TaskId::new("dep-1");
    let mut task = WorkerTask::new(TaskId::new("t1"), WorkerId::new(0), "t", "d");
    task.dependencies = vec![dep.clone()];
    assert_eq!(task.dependencies, vec![dep]);
}

#[test]
fn worker_task_has_no_dependencies_true_for_new() {
    let task = WorkerTask::new(TaskId::new("t1"), WorkerId::new(0), "t", "d");
    assert!(task.has_no_dependencies());
}

#[test]
fn worker_task_has_no_dependencies_false_after_setting_deps() {
    let mut task = WorkerTask::new(TaskId::new("t1"), WorkerId::new(0), "t", "d");
    task.dependencies = vec![TaskId::new("dep-1")];
    assert!(!task.has_no_dependencies());
}

#[test]
fn worker_task_with_task_type_works() {
    let task = WorkerTask::new(TaskId::new("t1"), WorkerId::new(0), "t", "d")
        .with_task_type(TaskType::CodeGeneration);
    assert_eq!(task.task_type, Some(TaskType::CodeGeneration));
}

#[test]
fn worker_task_with_complexity_works() {
    let task = WorkerTask::new(TaskId::new("t1"), WorkerId::new(0), "t", "d")
        .with_complexity(8);
    assert_eq!(task.complexity, 8);
}

#[test]
fn worker_task_with_affected_file_works() {
    let task = WorkerTask::new(TaskId::new("t1"), WorkerId::new(0), "t", "d")
        .with_affected_file("src/main.rs");
    assert_eq!(task.affected_files, vec!["src/main.rs"]);
}

// ---------------------------------------------------------------------------
// ExecutionPhase
// ---------------------------------------------------------------------------

#[test]
fn execution_phase_empty_is_constructible() {
    let phase = ExecutionPhase {
        phase_number: 0,
        tasks: vec![],
    };
    assert_eq!(phase.phase_number, 0);
    assert!(phase.tasks.is_empty());
}

#[test]
fn execution_phase_with_task_has_correct_len() {
    let task = WorkerTask::new(TaskId::new("t1"), WorkerId::new(0), "t", "d");
    let phase = ExecutionPhase {
        phase_number: 1,
        tasks: vec![task],
    };
    assert_eq!(phase.tasks.len(), 1);
}

// ---------------------------------------------------------------------------
// ExecutionPlan
// ---------------------------------------------------------------------------

#[test]
fn execution_plan_new_phases_is_empty() {
    let plan = ExecutionPlan::new();
    assert!(plan.phases.is_empty());
}

#[test]
fn execution_plan_new_contributing_developers_is_empty() {
    let plan = ExecutionPlan::new();
    assert!(plan.contributing_developers.is_empty());
}

#[test]
fn execution_plan_from_phases_has_correct_phase_count() {
    let phase = ExecutionPhase {
        phase_number: 0,
        tasks: vec![],
    };
    let plan = ExecutionPlan::from_phases(vec![phase]);
    assert_eq!(plan.phases.len(), 1);
}

#[test]
fn execution_plan_all_workers_single_task_returns_one() {
    let task = WorkerTask::new(TaskId::new("t1"), WorkerId::new(0), "t", "d");
    let phase = ExecutionPhase {
        phase_number: 0,
        tasks: vec![task],
    };
    let plan = ExecutionPlan::from_phases(vec![phase]);
    assert_eq!(plan.all_workers().len(), 1);
}

#[test]
fn execution_plan_validate_valid_plan_no_errors() {
    let task = WorkerTask::new(TaskId::new("t1"), WorkerId::new(0), "t", "d");
    let phase = ExecutionPhase {
        phase_number: 0,
        tasks: vec![task],
    };
    let plan = ExecutionPlan::from_phases(vec![phase]);
    assert!(plan.validate().is_empty());
}

#[test]
fn execution_plan_validate_duplicate_worker_id_returns_errors() {
    // Two tasks in the same phase sharing the same worker_id
    let task_a = WorkerTask::new(TaskId::new("t1"), WorkerId::new(0), "a", "a");
    let task_b = WorkerTask::new(TaskId::new("t2"), WorkerId::new(0), "b", "b");
    let phase = ExecutionPhase {
        phase_number: 0,
        tasks: vec![task_a, task_b],
    };
    let plan = ExecutionPlan::from_phases(vec![phase]);
    let errors = plan.validate();
    assert!(!errors.is_empty(), "expected validation errors for duplicate worker_id");
}

#[test]
fn execution_plan_validate_duplicate_task_id_returns_errors() {
    let task_a = WorkerTask::new(TaskId::new("same-id"), WorkerId::new(0), "a", "a");
    let task_b = WorkerTask::new(TaskId::new("same-id"), WorkerId::new(1), "b", "b");
    let phase = ExecutionPhase {
        phase_number: 0,
        tasks: vec![task_a, task_b],
    };
    let plan = ExecutionPlan::from_phases(vec![phase]);
    let errors = plan.validate();
    assert!(!errors.is_empty(), "expected validation errors for duplicate task_id");
}

// ---------------------------------------------------------------------------
// DAG functions
// ---------------------------------------------------------------------------

#[test]
fn validate_dag_empty_slice_returns_ok() {
    let i18n = i18n();
    assert!(validate_dag(&[], &i18n).is_ok());
}

#[test]
fn validate_dag_linear_dep_returns_ok() {
    let i18n = i18n();
    let task_a = WorkerTask::new(TaskId::new("a"), WorkerId::new(0), "A", "A desc");
    let task_b = WorkerTask::new(TaskId::new("b"), WorkerId::new(1), "B", "B desc")
        .with_dependency(TaskId::new("a"));
    assert!(validate_dag(&[task_a, task_b], &i18n).is_ok());
}

#[test]
fn validate_dag_cycle_returns_err() {
    let i18n = i18n();
    // a depends on b, b depends on a → cycle
    let task_a = WorkerTask::new(TaskId::new("a"), WorkerId::new(0), "A", "A desc")
        .with_dependency(TaskId::new("b"));
    let task_b = WorkerTask::new(TaskId::new("b"), WorkerId::new(1), "B", "B desc")
        .with_dependency(TaskId::new("a"));
    assert!(validate_dag(&[task_a, task_b], &i18n).is_err());
}

#[test]
fn detect_cycles_acyclic_returns_empty() {
    let task_a = WorkerTask::new(TaskId::new("a"), WorkerId::new(0), "A", "A desc");
    let task_b = WorkerTask::new(TaskId::new("b"), WorkerId::new(1), "B", "B desc")
        .with_dependency(TaskId::new("a"));
    let cycles = detect_cycles(&[task_a, task_b]);
    assert!(cycles.is_empty());
}

#[test]
fn detect_cycles_cycle_returns_non_empty() {
    let task_a = WorkerTask::new(TaskId::new("a"), WorkerId::new(0), "A", "A desc")
        .with_dependency(TaskId::new("b"));
    let task_b = WorkerTask::new(TaskId::new("b"), WorkerId::new(1), "B", "B desc")
        .with_dependency(TaskId::new("a"));
    let cycles = detect_cycles(&[task_a, task_b]);
    assert!(!cycles.is_empty());
}

#[test]
fn build_phases_linear_dep_returns_two_phases() {
    let i18n = i18n();
    let task_a = WorkerTask::new(TaskId::new("a"), WorkerId::new(0), "A", "A desc");
    let task_b = WorkerTask::new(TaskId::new("b"), WorkerId::new(1), "B", "B desc")
        .with_dependency(TaskId::new("a"));
    let phases = build_phases(&[task_a, task_b], &i18n)
        .expect("build_phases should succeed for acyclic graph");
    assert_eq!(phases.len(), 2, "expected 2 phases, got {}", phases.len());
}

#[test]
fn build_phases_phase_zero_contains_task_a() {
    let i18n = i18n();
    let task_a = WorkerTask::new(TaskId::new("a"), WorkerId::new(0), "A", "A desc");
    let task_b = WorkerTask::new(TaskId::new("b"), WorkerId::new(1), "B", "B desc")
        .with_dependency(TaskId::new("a"));
    let phases = build_phases(&[task_a, task_b], &i18n)
        .expect("build_phases should succeed");
    let phase0_ids: Vec<&str> = phases[0].tasks.iter().map(|t| t.task_id.as_str()).collect();
    assert!(
        phase0_ids.contains(&"a"),
        "phase 0 should contain task 'a', got: {phase0_ids:?}"
    );
}

#[test]
fn build_phases_phase_one_contains_task_b() {
    let i18n = i18n();
    let task_a = WorkerTask::new(TaskId::new("a"), WorkerId::new(0), "A", "A desc");
    let task_b = WorkerTask::new(TaskId::new("b"), WorkerId::new(1), "B", "B desc")
        .with_dependency(TaskId::new("a"));
    let phases = build_phases(&[task_a, task_b], &i18n)
        .expect("build_phases should succeed");
    let phase1_ids: Vec<&str> = phases[1].tasks.iter().map(|t| t.task_id.as_str()).collect();
    assert!(
        phase1_ids.contains(&"b"),
        "phase 1 should contain task 'b', got: {phase1_ids:?}"
    );
}

// ---------------------------------------------------------------------------
// WorkerQueue
// ---------------------------------------------------------------------------

/// Build a two-task execution plan: worker 0 (task "a") has no deps;
/// worker 1 (task "b") depends on "a".
fn two_task_plan() -> ExecutionPlan {
    let task_a = WorkerTask::new(TaskId::new("a"), WorkerId::new(0), "A", "A desc");
    let task_b = WorkerTask::new(TaskId::new("b"), WorkerId::new(1), "B", "B desc")
        .with_dependency(TaskId::new("a"));

    let phase0 = ExecutionPhase {
        phase_number: 0,
        tasks: vec![task_a],
    };
    let phase1 = ExecutionPhase {
        phase_number: 1,
        tasks: vec![task_b],
    };
    ExecutionPlan::from_phases(vec![phase0, phase1])
}

#[test]
fn worker_queue_from_plan_creates_without_panic() {
    let plan = two_task_plan();
    let _queue = WorkerQueue::from_plan(&plan, 2, 1);
}

#[test]
fn worker_queue_ready_worker_ids_includes_independent_task() {
    let plan = two_task_plan();
    let queue = WorkerQueue::from_plan(&plan, 2, 1);
    let ready = queue.ready_worker_ids();
    assert!(
        ready.contains(&WorkerId::new(0)),
        "worker 0 (no deps) should be ready; got: {ready:?}"
    );
}

#[test]
fn worker_queue_ready_worker_ids_excludes_blocked_task() {
    let plan = two_task_plan();
    let queue = WorkerQueue::from_plan(&plan, 2, 1);
    let ready = queue.ready_worker_ids();
    assert!(
        !ready.contains(&WorkerId::new(1)),
        "worker 1 (dep on worker 0) should NOT be ready initially; got: {ready:?}"
    );
}

#[test]
fn worker_queue_activate_worker_returns_ok() {
    let plan = two_task_plan();
    let mut queue = WorkerQueue::from_plan(&plan, 2, 1);
    let i18n = i18n();
    let result = queue.activate_worker(WorkerId::new(0), &i18n);
    assert!(result.is_ok(), "activate_worker returned Err: {:?}", result.err());
}

#[test]
fn worker_queue_complete_worker_returns_ok() {
    let plan = two_task_plan();
    let mut queue = WorkerQueue::from_plan(&plan, 2, 1);
    let i18n = i18n();
    queue.activate_worker(WorkerId::new(0), &i18n).unwrap();
    let result = queue.complete_worker(WorkerId::new(0), &i18n);
    assert!(result.is_ok(), "complete_worker returned Err: {:?}", result.err());
}

#[test]
fn worker_queue_after_completing_worker_0_worker_1_becomes_ready() {
    let plan = two_task_plan();
    let mut queue = WorkerQueue::from_plan(&plan, 2, 1);
    let i18n = i18n();
    queue.activate_worker(WorkerId::new(0), &i18n).unwrap();
    queue.complete_worker(WorkerId::new(0), &i18n).unwrap();
    let ready = queue.ready_worker_ids();
    assert!(
        ready.contains(&WorkerId::new(1)),
        "worker 1 should be ready after worker 0 completes; got: {ready:?}"
    );
}

#[test]
fn worker_queue_statistics_reflect_state_changes() {
    let plan = two_task_plan();
    let mut queue = WorkerQueue::from_plan(&plan, 2, 1);
    let i18n = i18n();

    // Initially: 2 total, 1 pending (worker 0), 1 blocked (worker 1)
    let stats = queue.statistics();
    assert_eq!(stats.total, 2);
    assert_eq!(stats.pending, 1);
    assert_eq!(stats.blocked, 1);
    assert_eq!(stats.active, 0);
    assert_eq!(stats.completed, 0);

    // Activate worker 0
    queue.activate_worker(WorkerId::new(0), &i18n).unwrap();
    let stats = queue.statistics();
    assert_eq!(stats.active, 1);
    assert_eq!(stats.pending, 0);

    // Complete worker 0
    queue.complete_worker(WorkerId::new(0), &i18n).unwrap();
    let stats = queue.statistics();
    assert_eq!(stats.completed, 1);
    assert_eq!(stats.active, 0);
    // worker 1 should now be pending (unblocked)
    assert_eq!(stats.pending, 1);
    assert_eq!(stats.blocked, 0);
}
