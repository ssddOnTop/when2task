//! Simple integration tests for when2task library
//! Tests the public API as an external user would use it

use when2task::{Task, TaskExecutor};
use std::time::{Duration, Instant};
use tokio::time::sleep;

/// Test basic task creation and execution
#[tokio::test]
async fn test_basic_task_execution() {
    let mut executor = TaskExecutor::new();
    
    let task = Task::new(
        async { Ok::<String, String>("Hello, World!".to_string()) },
        vec![],
    );
    
    executor.add_task(task);
    
    let result = executor.execute().await.expect("Execution should succeed");
    
    assert_eq!(result.total_tasks, 1);
    assert_eq!(result.successful_tasks, 1);
    assert_eq!(result.failed_tasks, 0);
    assert!(result.all_successful());
    assert_eq!(result.steps.len(), 1);
}

/// Test builder pattern for adding tasks
#[tokio::test]
async fn test_builder_pattern() {
    let task1 = Task::new(
        async { Ok::<i32, String>(42) },
        vec![],
    );
    
    let task2 = Task::new(
        async { Ok::<i32, String>(84) },
        vec![],
    );
    
    let executor = TaskExecutor::new()
        .insert(task1)
        .insert(task2);
    
    let result = executor.execute().await.expect("Execution should succeed");
    
    assert_eq!(result.total_tasks, 2);
    assert_eq!(result.successful_tasks, 2);
    assert_eq!(result.failed_tasks, 0);
    assert_eq!(result.steps.len(), 1); // Both should run in parallel
}

/// Test that independent tasks run in parallel
#[tokio::test]
async fn test_parallel_execution() {
    let mut executor = TaskExecutor::new();
    
    // Create three tasks that each take 50ms
    for i in 0..3 {
        let task = Task::new(
            async move {
                sleep(Duration::from_millis(50)).await;
                Ok::<i32, String>(i)
            },
            vec![],
        );
        executor.add_task(task);
    }
    
    let start = Instant::now();
    let result = executor.execute().await.expect("Execution should succeed");
    let duration = start.elapsed();
    
    // Should complete in roughly 50ms (parallel) rather than 150ms (sequential)
    assert!(duration < Duration::from_millis(100), "Tasks should run in parallel");
    assert_eq!(result.total_tasks, 3);
    assert_eq!(result.successful_tasks, 3);
    assert_eq!(result.steps.len(), 1); // All in one step
}

/// Test sequential execution with dependencies
#[tokio::test]
async fn test_sequential_execution() {
    let mut executor = TaskExecutor::new();
    
    let task1 = Task::new(
        async { 
            sleep(Duration::from_millis(30)).await;
            Ok::<String, String>("Task 1".to_string())
        },
        vec![],
    );
    let task1_id = *task1.id();
    
    let task2 = Task::new(
        async { 
            sleep(Duration::from_millis(30)).await;
            Ok::<String, String>("Task 2".to_string())
        },
        vec![task1_id],
    );
    
    executor.add_task(task1);
    executor.add_task(task2);
    
    let start = Instant::now();
    let result = executor.execute().await.expect("Execution should succeed");
    let duration = start.elapsed();
    
    // Should take roughly 60ms (sequential)
    assert!(duration >= Duration::from_millis(50), "Tasks should run sequentially");
    assert_eq!(result.total_tasks, 2);
    assert_eq!(result.successful_tasks, 2);
    assert_eq!(result.steps.len(), 2); // Two sequential steps
}

/// Test mixed parallel and sequential execution
#[tokio::test]
async fn test_mixed_execution() {
    let mut executor = TaskExecutor::new();
    
    // Step 1: Two independent tasks
    let task1 = Task::new(
        async { Ok::<String, String>("Independent 1".to_string()) },
        vec![],
    );
    let task1_id = *task1.id();
    
    let task2 = Task::new(
        async { Ok::<String, String>("Independent 2".to_string()) },
        vec![],
    );
    let task2_id = *task2.id();
    
    // Step 2: Task that depends on both
    let task3 = Task::new(
        async { Ok::<String, String>("Dependent on both".to_string()) },
        vec![task1_id, task2_id],
    );
    
    executor.add_task(task1);
    executor.add_task(task2);
    executor.add_task(task3);
    
    let result = executor.execute().await.expect("Execution should succeed");
    
    assert_eq!(result.total_tasks, 3);
    assert_eq!(result.successful_tasks, 3);
    assert_eq!(result.steps.len(), 2); // Two steps
    
    // Step 1 should have 2 tasks (task1, task2)
    assert_eq!(result.steps[0].len(), 2);
    // Step 2 should have 1 task (task3)
    assert_eq!(result.steps[1].len(), 1);
}

/// Test task execution failure
#[tokio::test]
async fn test_task_failure() {
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
    
    let result = executor.execute().await.expect("Execution should complete despite task failure");
    
    assert_eq!(result.total_tasks, 2);
    assert_eq!(result.successful_tasks, 1);
    assert_eq!(result.failed_tasks, 1);
    assert!(!result.all_successful());
    
    // Verify we can access both successful and failed results
    let successful_results: Vec<_> = result.successful_results().collect();
    let failed_results: Vec<_> = result.failed_results().collect();
    
    assert_eq!(successful_results.len(), 1);
    assert_eq!(failed_results.len(), 1);
}

/// Test empty executor
#[tokio::test]
async fn test_empty_executor() {
    let executor = TaskExecutor::<String, String>::new();
    
    let result = executor.execute().await.expect("Empty executor should succeed");
    
    assert_eq!(result.total_tasks, 0);
    assert_eq!(result.successful_tasks, 0);
    assert_eq!(result.failed_tasks, 0);
    assert!(result.all_successful()); // Vacuously true
    assert_eq!(result.steps.len(), 0);
}

/// Test utility methods
#[tokio::test]
async fn test_executor_utility_methods() {
    let mut executor = TaskExecutor::new();
    
    assert_eq!(executor.task_count(), 0);
    
    let task1 = Task::new(
        async { Ok::<String, String>("test".to_string()) },
        vec![],
    );
    let task1_id = *task1.id();
    
    executor.add_task(task1);
    
    assert_eq!(executor.task_count(), 1);
    assert!(executor.has_task(&task1_id));
    
    // Test blueprint creation
    let blueprint = executor.create_blueprint().expect("Blueprint creation should succeed");
    assert_eq!(blueprint.step_count(), 1);
}