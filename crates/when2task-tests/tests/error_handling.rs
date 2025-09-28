//! Simple error handling tests for when2task library
//! Tests various error scenarios and edge cases

use std::collections::HashMap;
use when2task::{Blueprint, BlueprintError, ExecutionError, Task, TaskExecutor};

/// Test task execution failure
#[tokio::test]
async fn test_task_execution_failure() {
    let mut executor = TaskExecutor::new();

    let failing_task = Task::new(
        async { Err::<String, String>("Task failed".to_string()) },
        vec![],
    );

    let successful_task = Task::new(
        async { Ok::<String, String>("Task succeeded".to_string()) },
        vec![],
    );

    executor.add_task(failing_task);
    executor.add_task(successful_task);

    let result = executor
        .execute()
        .await
        .expect("Execution should complete despite task failure");

    assert_eq!(result.total_tasks, 2);
    assert_eq!(result.successful_tasks, 1);
    assert_eq!(result.failed_tasks, 1);
    assert!(!result.all_successful());
}

/// Test missing dependency detection
#[tokio::test]
async fn test_missing_dependency_detection() {
    let mut executor = TaskExecutor::new();

    let task1 = Task::new(async { Ok::<String, String>("Task 1".to_string()) }, vec![]);

    let fake_task_id = when2task::TaskId::generate();

    let task2 = Task::new(
        async { Ok::<String, String>("Task 2".to_string()) },
        vec![fake_task_id],
    );
    let task2_id = *task2.id();

    executor.add_task(task1);
    executor.add_task(task2);

    let result = executor.execute().await;

    match result {
        Err(ExecutionError::BlueprintError(BlueprintError::MissingDependency(id, _))) => {
            assert_eq!(id, task2_id);
        }
        _ => panic!("Expected missing dependency error"),
    }
}

/// Test empty executor
#[tokio::test]
async fn test_empty_executor() {
    let executor = TaskExecutor::<String, String>::new();

    let result = executor
        .execute()
        .await
        .expect("Empty executor should succeed");

    assert_eq!(result.total_tasks, 0);
    assert_eq!(result.successful_tasks, 0);
    assert_eq!(result.failed_tasks, 0);
    assert!(result.all_successful());
    assert_eq!(result.steps.len(), 0);
}

/// Test blueprint creation with invalid dependencies
#[tokio::test]
async fn test_blueprint_creation_errors() {
    let mut tasks = HashMap::new();

    let task1 = Task::new(async { Ok::<String, String>("Task 1".to_string()) }, vec![]);
    let task1_id = *task1.id();

    // Create task with dependency on non-existent task
    let fake_id = when2task::TaskId::generate();
    let task2 = Task::new(
        async { Ok::<String, String>("Task 2".to_string()) },
        vec![fake_id],
    );

    tasks.insert(task1_id, task1);
    tasks.insert(*task2.id(), task2);

    let result = Blueprint::from_tasks(&tasks);

    match result {
        Err(BlueprintError::MissingDependency(_id, _)) => {

        }
        _ => panic!("Expected missing dependency error"),
    }
}

/// Test blueprint validation edge cases
#[tokio::test]
async fn test_blueprint_edge_cases() {
    // Test with empty task map
    let empty_tasks: HashMap<when2task::TaskId, when2task::Task<String, String>> = HashMap::new();
    let blueprint = Blueprint::from_tasks(&empty_tasks).expect("Empty blueprint should succeed");
    assert_eq!(blueprint.step_count(), 0);

    // Test single task
    let mut single_task_map = HashMap::new();
    let task = Task::new(async { Ok::<String, String>("Single".to_string()) }, vec![]);
    let task_id = *task.id();
    single_task_map.insert(task_id, task);

    let blueprint =
        Blueprint::from_tasks(&single_task_map).expect("Single task blueprint should succeed");
    assert_eq!(blueprint.step_count(), 1);

    let step_tasks = blueprint.tasks_at_step(0).expect("Step 0 should exist");
    assert_eq!(step_tasks.len(), 1);
    assert_eq!(step_tasks[0], task_id);
}
