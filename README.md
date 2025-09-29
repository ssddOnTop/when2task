# When2Task: Optimized Async Task Execution Framework

**When2Task** is a high-performance Rust library for executing async tasks with automatic dependency resolution and
optimal parallelization. It uses a blueprint-based execution model that transforms task dependencies into an optimized
execution plan.

# Plans

- [x] Create async task executor with ability to plan execute tasks asynchronously or execute tasks in parallel and threat
the task group as async.

- [] Create truly parallel task executor

- ~~[] Ability to pass outputs as inputs to dependent tasks~~

- [x] Ability to return a map of Task ID and task results after execution.
