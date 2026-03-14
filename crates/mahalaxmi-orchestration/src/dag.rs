// SPDX-License-Identifier: MIT
// Copyright 2026 ThriveTech Services LLC
use mahalaxmi_core::error::MahalaxmiError;
use mahalaxmi_core::i18n::messages::keys;
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::types::TaskId;
use mahalaxmi_core::MahalaxmiResult;
use std::collections::{HashMap, HashSet, VecDeque};

use crate::models::plan::{ExecutionPhase, WorkerTask};

/// Validate that a set of tasks forms a valid DAG (no circular dependencies).
///
/// Returns `Ok(())` if the graph is valid, or an error describing the cycle.
pub fn validate_dag(tasks: &[WorkerTask], i18n: &I18nService) -> MahalaxmiResult<()> {
    let cycles = detect_cycles(tasks);
    if cycles.is_empty() {
        Ok(())
    } else {
        let cycle_desc = cycles
            .iter()
            .map(|c| {
                c.iter()
                    .map(|id| id.as_str())
                    .collect::<Vec<_>>()
                    .join(" -> ")
            })
            .collect::<Vec<_>>()
            .join("; ");
        Err(MahalaxmiError::orchestration(
            i18n,
            keys::orchestration::CIRCULAR_DEPENDENCY,
            &[("cycle", &cycle_desc)],
        ))
    }
}

/// Detect circular dependencies in a set of tasks.
///
/// Returns a list of cycles found, where each cycle is a list of task IDs.
/// Returns an empty vec if the graph is acyclic.
pub fn detect_cycles(tasks: &[WorkerTask]) -> Vec<Vec<TaskId>> {
    let mut adjacency: HashMap<&TaskId, Vec<&TaskId>> = HashMap::new();
    let mut all_ids: HashSet<&TaskId> = HashSet::new();

    for task in tasks {
        all_ids.insert(&task.task_id);
        adjacency.entry(&task.task_id).or_default();
        for dep in &task.dependencies {
            adjacency.entry(dep).or_default();
            if let Some(neighbors) = adjacency.get_mut(&task.task_id) {
                neighbors.push(dep);
            }
            all_ids.insert(dep);
        }
    }

    let mut cycles = Vec::new();
    let mut visited = HashSet::new();
    let mut rec_stack = HashSet::new();
    let mut path = Vec::new();

    for id in &all_ids {
        if !visited.contains(*id) {
            dfs_detect_cycle(
                id,
                &adjacency,
                &mut visited,
                &mut rec_stack,
                &mut path,
                &mut cycles,
            );
        }
    }

    cycles
}

/// DFS helper for cycle detection.
fn dfs_detect_cycle<'a>(
    node: &'a TaskId,
    adjacency: &HashMap<&'a TaskId, Vec<&'a TaskId>>,
    visited: &mut HashSet<&'a TaskId>,
    rec_stack: &mut HashSet<&'a TaskId>,
    path: &mut Vec<&'a TaskId>,
    cycles: &mut Vec<Vec<TaskId>>,
) {
    visited.insert(node);
    rec_stack.insert(node);
    path.push(node);

    if let Some(neighbors) = adjacency.get(node) {
        for neighbor in neighbors {
            if !visited.contains(*neighbor) {
                dfs_detect_cycle(neighbor, adjacency, visited, rec_stack, path, cycles);
            } else if rec_stack.contains(*neighbor) {
                // Found a cycle — extract it from the path
                if let Some(start) = path.iter().position(|n| *n == *neighbor) {
                    let cycle: Vec<TaskId> = path[start..].iter().map(|id| (*id).clone()).collect();
                    cycles.push(cycle);
                }
            }
        }
    }

    path.pop();
    rec_stack.remove(node);
}

/// Produce a topological ordering of the tasks.
///
/// Returns the task IDs in dependency order (dependencies come first).
/// Returns an error if the graph contains cycles.
pub fn topological_sort(tasks: &[WorkerTask], i18n: &I18nService) -> MahalaxmiResult<Vec<TaskId>> {
    validate_dag(tasks, i18n)?;

    let mut in_degree: HashMap<TaskId, usize> = HashMap::new();
    let mut dependents: HashMap<TaskId, Vec<TaskId>> = HashMap::new();

    for task in tasks {
        in_degree.entry(task.task_id.clone()).or_insert(0);
        for dep in &task.dependencies {
            dependents
                .entry(dep.clone())
                .or_default()
                .push(task.task_id.clone());
            *in_degree.entry(task.task_id.clone()).or_insert(0) += 1;
        }
    }

    let mut queue: VecDeque<TaskId> = in_degree
        .iter()
        .filter(|(_, &deg)| deg == 0)
        .map(|(id, _)| id.clone())
        .collect();

    let mut result = Vec::new();
    while let Some(id) = queue.pop_front() {
        result.push(id.clone());
        if let Some(deps) = dependents.get(&id) {
            for dep in deps {
                if let Some(deg) = in_degree.get_mut(dep) {
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push_back(dep.clone());
                    }
                }
            }
        }
    }

    Ok(result)
}

/// Build execution phases from a set of tasks using topological ordering.
///
/// Tasks are grouped into phases where all tasks in a phase have their
/// dependencies satisfied by tasks in earlier phases.
pub fn build_phases(
    tasks: &[WorkerTask],
    i18n: &I18nService,
) -> MahalaxmiResult<Vec<ExecutionPhase>> {
    validate_dag(tasks, i18n)?;

    let task_map: HashMap<TaskId, &WorkerTask> =
        tasks.iter().map(|t| (t.task_id.clone(), t)).collect();

    let mut assigned: HashSet<TaskId> = HashSet::new();
    let mut phases = Vec::new();
    let mut remaining: HashSet<TaskId> = tasks.iter().map(|t| t.task_id.clone()).collect();

    let mut phase_number = 0u32;
    while !remaining.is_empty() {
        let ready: Vec<TaskId> = remaining
            .iter()
            .filter(|id| {
                task_map
                    .get(*id)
                    .map(|t| t.dependencies.iter().all(|d| assigned.contains(d)))
                    .unwrap_or(false)
            })
            .cloned()
            .collect();

        if ready.is_empty() {
            break; // Should not happen if validate_dag passed
        }

        let phase_tasks: Vec<WorkerTask> = ready
            .iter()
            .filter_map(|id| task_map.get(id).map(|t| (*t).clone()))
            .collect();

        for id in &ready {
            remaining.remove(id);
            assigned.insert(id.clone());
        }

        phases.push(ExecutionPhase {
            phase_number,
            tasks: phase_tasks,
        });
        phase_number += 1;
    }

    Ok(phases)
}
