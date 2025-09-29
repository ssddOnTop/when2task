# when2task

A lightweight, dependency-aware task execution library for Rust that allows you to define tasks with dependencies and execute them in the correct order.

## Features

- **Dependency Management**: Define tasks with dependencies on other tasks
- **Automatic Scheduling**: Tasks are automatically executed in dependency order
- **Concurrent Execution**: Independent tasks run concurrently within each execution step
- **Flexible Execution Modes**: Choose between true async or pseudo async execution
- **Type Safety**: Full generic support for task results and errors

## Example

```rust
use when2task::{Task, TaskExecutor, ExecutionMode, Dependency};
use std::time::{Instant, Duration};

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
        Dependency::from([task_a_id, task_b_id])
    );
    
    let executor = TaskExecutor::new(ExecutionMode::true_async())
        .insert(task_a)
        .insert(task_b)
        .insert(task_c);
    
    let result = executor.execute().await.unwrap();
    
    println!("Total execution time: {:?}", start.elapsed());
    println!("Tasks completed in {} steps", result.steps.len());
    // Output shows ~200ms total (150ms for step 1 + 50ms for step 2)
    // demonstrating concurrent execution of A & B, then C
}
```

## Core Concepts

- **Task**: A unit of work represented as a future that returns `Result<T, E>`
- **Dependency**: Specifies which tasks must complete before a task can run
- **TaskExecutor**: Manages and executes tasks in dependency order
- **ExecutionMode**: Determines how tasks are scheduled (direct execution vs spawned)
- **ExecutionResult**: Contains the results of all executed tasks organized by execution steps

## License

This project is licensed under the MIT License - see the LICENSE file for details.
