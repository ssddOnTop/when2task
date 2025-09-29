use std::time::{Duration, Instant};
use when2task::{Dependency, ExecutionMode, Task, TaskExecutor};

#[tokio::main]
async fn main() {
    let start = Instant::now();

    // Independent tasks that run concurrently
    let task_a = Task::new_independent(async {
        tokio::time::sleep(Duration::from_millis(100)).await;
        println!("Task A completed after 100ms");
        Ok::<&str, ()>("A")
    });

    let task_b = Task::new_independent(async {
        tokio::time::sleep(Duration::from_millis(150)).await;
        println!("Task B completed after 150ms");
        Ok::<&str, ()>("B")
    });

    let task_a_id = *task_a.id();
    let task_b_id = *task_b.id();

    // Dependent task that waits for both A and B
    let task_c = Task::new(
        async {
            tokio::time::sleep(Duration::from_millis(50)).await;
            println!("Task C completed after both A and B (50ms additional)");
            Ok::<&str, ()>("C")
        },
        Dependency::from([task_a_id, task_b_id]),
    );

    let executor = TaskExecutor::new(ExecutionMode::true_async())
        .insert(task_a)
        .insert(task_b)
        .insert(task_c);

    let result = executor.execute().await.unwrap();

    assert!(start.elapsed().as_millis() < 210);
    assert_eq!(2, result.steps.len());
    // Output shows ~200ms total (150ms for step 1 + 50ms for step 2)
    // demonstrating concurrent execution of A & B, then C
}
