//! Basic usage example demonstrating the optimized async task execution system
//!
//! This example shows how to:
//! 1. Create tasks with and without dependencies
//! 2. Build an execution plan using the blueprint system
//! 3. Execute tasks optimally (parallel execution for independent tasks)
//! 4. Handle results and errors

use std::time::Duration;
use tokio::time::sleep;
use when2task::{Task, TaskExecutor};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting optimized async task execution demo");

    // Create a task executor
    let mut executor = TaskExecutor::new();

    // Create some independent tasks that can run in parallel
    let task1 = create_fetch_task("Database query", 200);
    let task1_id = *task1.id();

    let task2 = create_fetch_task("API call", 300);
    let task2_id = *task2.id();

    let task3 = create_fetch_task("File read", 150);
    let task3_id = *task3.id();

    // Create tasks that depend on the above tasks
    let task4 = create_processing_task("Process DB + API", 100, vec![task1_id, task2_id]);
    let task4_id = *task4.id();

    let task5 = create_processing_task("Process File", 80, vec![task3_id]);
    let task5_id = *task5.id();

    // Create a final task that depends on all processing
    let task6 = create_processing_task("Final aggregation", 50, vec![task4_id, task5_id]);

    // Add all tasks to executor
    executor.add_task(task1);
    executor.add_task(task2);
    executor.add_task(task3);
    executor.add_task(task4);
    executor.add_task(task5);
    executor.add_task(task6);

    println!("📋 Created {} tasks", executor.task_count());

    // Create and display the execution blueprint
    let blueprint = executor.create_blueprint()?;
    println!("\n📊 Execution Plan:");
    for (step_idx, step) in blueprint.steps.iter().enumerate() {
        println!(
            "  Step {}: {} tasks in parallel",
            step_idx + 1,
            step.tasks.len()
        );
    }

    println!("\n⏱️ Executing tasks...");
    let start_time = std::time::Instant::now();

    // Execute all tasks
    let result = executor.execute().await?;

    let duration = start_time.elapsed();
    println!("\n✅ Execution completed in {:?}", duration);

    // Display results
    println!("\n📈 Results:");
    println!("  Total tasks: {}", result.total_tasks);
    println!("  Successful: {}", result.successful_tasks);
    println!("  Failed: {}", result.failed_tasks);
    println!("  Steps executed: {}", result.steps.len());

    // Show step-by-step results
    for (step_idx, step_results) in result.steps.iter().enumerate() {
        println!("\n  Step {} results:", step_idx + 1);
        for task_result in step_results {
            match &task_result.result {
                Ok(value) => println!("    ✅ Task {:?}: {}", task_result.task_id, value),
                Err(error) => println!("    ❌ Task {:?}: {}", task_result.task_id, error),
            }
        }
    }

    if result.all_successful() {
        println!("\n🎉 All tasks completed successfully!");
    } else {
        println!("\n⚠️ Some tasks failed.");
    }

    println!("\n💡 Note: Independent tasks (steps 1) ran in parallel, saving time!");
    println!(
        "   Sequential execution would take ~880ms, but parallel execution took ~{:?}",
        duration
    );

    Ok(())
}

/// Creates a simulated fetch task (database, API, file operations)
fn create_fetch_task(name: &'static str, delay_ms: u64) -> Task<'static, String, String> {
    let future = async move {
        println!("  🔄 Starting: {}", name);
        sleep(Duration::from_millis(delay_ms)).await;
        println!("  ✅ Completed: {}", name);
        Ok(format!("{} result", name))
    };
    Task::new(future, vec![])
}

/// Creates a simulated processing task that depends on other tasks
fn create_processing_task(
    name: &'static str,
    delay_ms: u64,
    dependencies: Vec<when2task::TaskId>,
) -> Task<'static, String, String> {
    let dep_count = dependencies.len();
    let future = async move {
        println!("  🔄 Starting: {} (depends on {} tasks)", name, dep_count);
        sleep(Duration::from_millis(delay_ms)).await;
        println!("  ✅ Completed: {}", name);
        Ok(format!("{} result", name))
    };
    Task::new(future, dependencies)
}
