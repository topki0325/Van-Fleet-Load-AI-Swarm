# C Compiler Environment Management

## Overview

C Compiler Environment Management provides automatic discovery and intelligent scheduling of GCC (GNU Compiler Collection) instances. It supports both round-robin and parallel compilation strategies to maximize utilization of multiple GCC instances and reduce compilation time.

## Features

### 1. Automatic GCC Instance Discovery
- **System-wide Search**: Automatically discovers all GCC instances on the system
- **Path Detection**: Finds GCC in standard system paths
- **Version Detection**: Identifies GCC version for each instance
- **Status Tracking**: Monitors availability and current tasks of each instance

### 2. Round-Robin Compilation
- **Load Balancing**: Distributes compilation tasks evenly across GCC instances
- **Fair Scheduling**: Ensures each instance gets fair share of tasks
- **Automatic Selection**: Automatically selects next available GCC instance
- **Task Tracking**: Tracks current task for each instance

### 3. Parallel Compilation
- **Concurrent Execution**: Compiles multiple files simultaneously
- **Max Concurrency Control**: Configurable maximum number of parallel tasks
- **Result Aggregation**: Collects results from all parallel tasks
- **Error Handling**: Handles errors per task without affecting others

### 4. Compiler Status Monitoring
- **Real-time Status**: View current status of all GCC instances
- **Task Tracking**: Monitor active and completed tasks
- **Performance Metrics**: Track compilation times and success rates

## Architecture

### GCC Instance
Each GCC instance is represented by:
- **ID**: Unique identifier for the instance
- **Path**: Full path to the GCC executable
- **Availability**: Whether the instance is available for new tasks
- **Current Task**: Currently assigned task (if any)

### Compilation Task
Each compilation task includes:
- **Task ID**: Unique identifier for the task
- **Source Files**: List of C source files to compile
- **Output Path**: Path for the compiled binary
- **Compiler Flags**: Additional compiler flags (e.g., -Wall, -Wextra)
- **Include Paths**: Additional include directories
- **Optimization Level**: Optimization level (O0, O1, O2, O3, Os)

### Compilation Result
Each compilation result includes:
- **Task ID**: Identifier of the task
- **Success**: Whether compilation succeeded
- **Binary Path**: Path to compiled binary (if successful)
- **Output**: Compiler output (stdout)
- **Error Output**: Compiler errors (stderr)
- **Compilation Time**: Time taken to compile
- **GCC Instance ID**: Which GCC instance was used

## Web Interface Usage

### GCC Management
1. Click "List GCC Instances" to view all discovered GCC instances
2. Click "Get Status" to view current status and statistics

### Round-Robin Compilation
1. Enter source files (comma-separated, e.g., "main.c,utils.c")
2. Enter output path (e.g., "target/myapp")
3. Select optimization level:
   - **O0**: No optimization (fastest compilation)
   - **O1**: Basic optimization
   - **O2**: Recommended optimization (default)
   - **O3**: Aggressive optimization (slowest compilation)
   - **Os**: Size optimization
4. Click "Compile (Round Robin)" to compile

### Parallel Compilation
1. Enter tasks as JSON array:
```json
[
  {
    "task_id": "task1",
    "source_files": ["file1.c"],
    "output_path": "target/file1",
    "compiler_flags": ["-Wall"],
    "include_paths": [],
    "optimization_level": "2"
  },
  {
    "task_id": "task2",
    "source_files": ["file2.c"],
    "output_path": "target/file2",
    "compiler_flags": ["-Wall", "-Wextra"],
    "include_paths": ["include"],
    "optimization_level": "2"
  }
]
```
2. Click "Compile (Parallel)" to compile all tasks simultaneously

## Code Usage Examples

### Initialize C Compiler Scheduler
```rust
use vangriten_ai_swarm::backend::CCompilationScheduler;

// Create scheduler with max 4 parallel tasks
let scheduler = CCompilationScheduler::new(4).await?;

// List all GCC instances
let instances = scheduler.list_gcc_instances().await?;
for instance in instances {
    println!("GCC: {} (Version: {})", instance.gcc_path, instance.version);
}
```

### Round-Robin Compilation
```rust
use vangriten_ai_swarm::backend::c_compiler::CCompilationTask;

let task = CCompilationTask {
    task_id: uuid::Uuid::new_v4().to_string(),
    source_files: vec![
        std::path::PathBuf::from("main.c"),
        std::path::PathBuf::from("utils.c"),
    ],
    output_path: std::path::PathBuf::from("target/myapp"),
    compiler_flags: vec!["-Wall".to_string(), "-Wextra".to_string()],
    include_paths: vec![std::path::PathBuf::from("include")],
    optimization_level: "2".to_string(),
};

let result = scheduler.compile_round_robin(task).await?;
if result.success {
    println!("Compilation successful: {:?}", result.binary_path);
} else {
    println!("Compilation failed: {}", result.error_output);
}
```

### Parallel Compilation
```rust
use vangriten_ai_swarm::backend::c_compiler::CCompilationTask;

let tasks = vec![
    CCompilationTask {
        task_id: "task1".to_string(),
        source_files: vec![std::path::PathBuf::from("file1.c")],
        output_path: std::path::PathBuf::from("target/file1"),
        compiler_flags: vec!["-Wall".to_string()],
        include_paths: vec![],
        optimization_level: "2".to_string(),
    },
    CCompilationTask {
        task_id: "task2".to_string(),
        source_files: vec![std::path::PathBuf::from("file2.c")],
        output_path: std::path::PathBuf::from("target/file2"),
        compiler_flags: vec!["-Wall".to_string()],
        include_paths: vec![],
        optimization_level: "2".to_string(),
    },
];

let results = scheduler.compile_parallel(tasks).await;
for result in results {
    match result {
        Ok(compilation_result) => {
            if compilation_result.success {
                println!("Task {} succeeded: {:?}", 
                         compilation_result.task_id, compilation_result.binary_path);
            } else {
                println!("Task {} failed: {}", 
                         compilation_result.task_id, compilation_result.error_output);
            }
        }
        Err(e) => {
            println!("Task error: {}", e);
        }
    }
}
```

### Get Compiler Status
```rust
let status = scheduler.get_compiler_status().await;
println!("Total GCC instances: {}", status.total_instances);
println!("Available: {}", status.available_instances);
println!("Busy: {}", status.busy_instances);
println!("Max parallel tasks: {}", status.max_parallel_tasks);
```

## Optimization Levels

### O0 - No Optimization
- **Description**: No optimization, fastest compilation
- **Use Case**: Debug builds, development
- **Trade-off**: Fast compilation, slower execution

### O1 - Basic Optimization
- **Description**: Basic optimizations without increasing code size
- **Use Case**: Development builds
- **Trade-off**: Balanced compilation and execution speed

### O2 - Recommended Optimization
- **Description**: Recommended optimization level for most cases
- **Use Case**: Production builds (default)
- **Trade-off**: Good balance of compilation and execution speed

### O3 - Aggressive Optimization
- **Description**: Aggressive optimizations
- **Use Case**: Performance-critical applications
- **Trade-off**: Slowest compilation, fastest execution

### Os - Size Optimization
- **Description**: Optimize for code size
- **Use Case**: Embedded systems, memory-constrained environments
- **Trade-off**: Smaller binary size, potentially slower execution

## Best Practices

### 1. Choosing Compilation Strategy
- **Round-Robin**: Use for small to medium projects with few files
- **Parallel**: Use for large projects with many independent files

### 2. Setting Max Parallel Tasks
- Set to number of CPU cores for optimal performance
- Reduce if system has limited memory
- Increase for systems with high I/O performance

### 3. Compiler Flags
- Always use `-Wall` to enable all warnings
- Use `-Wextra` for additional warnings
- Use `-Werror` to treat warnings as errors in production
- Use `-O2` for production builds (default)
- Use `-O0` for debug builds

### 4. Include Paths
- Organize headers in logical directories
- Use relative paths when possible
- Avoid circular dependencies

### 5. Error Handling
- Always check compilation results
- Review error output for common issues
- Fix warnings before production

## Performance Optimization

### 1. Parallel Compilation
- Compile independent files in parallel
- Use appropriate max parallel tasks
- Monitor system resources during compilation

### 2. Incremental Compilation
- Only recompile changed files
- Use dependency tracking
- Implement build systems (Make, CMake)

### 3. Caching
- Cache compiled object files
- Use ccache for faster rebuilds
- Implement precompiled headers

### 4. Link Time Optimization (LTO)
- Enable LTO with `-flto` flag
- Improves performance across compilation units
- Increases compilation time

## Troubleshooting

### No GCC Instances Found
**Problem**: No GCC instances discovered

**Solutions**:
1. Install GCC: `sudo apt install gcc` (Linux), `brew install gcc` (macOS)
2. Check if GCC is in system PATH
3. Verify GCC installation: `gcc --version`
4. Manually specify GCC path if needed

### Compilation Failed
**Problem**: Compilation errors

**Solutions**:
1. Check source code for syntax errors
2. Review compiler error messages
3. Verify include paths are correct
4. Check compiler flags compatibility

### Slow Compilation
**Problem**: Compilation taking too long

**Solutions**:
1. Increase max parallel tasks
2. Use parallel compilation for multiple files
3. Reduce optimization level during development
4. Use faster storage (SSD)

### Out of Memory
**Problem**: System runs out of memory during compilation

**Solutions**:
1. Reduce max parallel tasks
2. Compile files sequentially
3. Close other applications
4. Increase system memory

## API Reference

### CCompilationScheduler
```rust
pub struct CCompilationScheduler {
    // Internal implementation
}
```

#### Methods
- `new(max_parallel_tasks: usize) -> Result<Self, VgaError>` - Create scheduler
- `list_gcc_instances(&self) -> Vec<CGccInstance>` - List GCC instances
- `get_compiler_status(&self) -> CCompilerStatus` - Get compiler status
- `compile_round_robin(&self, task: CCompilationTask) -> Result<CCompilationResult, VgaError>` - Round-robin compile
- `compile_parallel(&self, tasks: Vec<CCompilationTask>) -> Vec<Result<CCompilationResult, VgaError>>` - Parallel compile

### CGccInstance
```rust
pub struct CGccInstance {
    pub id: String,
    pub gcc_path: PathBuf,
    pub version: Option<String>,
    pub is_available: bool,
    pub current_task: Option<String>,
}
```

### CCompilationTask
```rust
pub struct CCompilationTask {
    pub task_id: String,
    pub source_files: Vec<PathBuf>,
    pub output_path: PathBuf,
    pub compiler_flags: Vec<String>,
    pub include_paths: Vec<PathBuf>,
    pub optimization_level: String,
}
```

### CCompilationResult
```rust
pub struct CCompilationResult {
    pub task_id: String,
    pub success: bool,
    pub binary_path: Option<PathBuf>,
    pub output: String,
    pub error_output: String,
    pub compilation_time: Duration,
    pub gcc_instance_id: String,
}
```

### CCompilerStatus
```rust
pub struct CCompilerStatus {
    pub total_instances: usize,
    pub available_instances: usize,
    pub busy_instances: usize,
    pub max_parallel_tasks: usize,
}
```

## Example: Hello World

### Source Code (hello.c)
```c
#include <stdio.h>

int main() {
    printf("Hello from Vangriten AI Swarm C Compiler!\n");
    return 0;
}
```

### Compilation
```rust
let task = CCompilationTask {
    task_id: "hello-world".to_string(),
    source_files: vec![PathBuf::from("hello.c")],
    output_path: PathBuf::from("target/hello"),
    compiler_flags: vec!["-Wall".to_string()],
    include_paths: vec![],
    optimization_level: "2".to_string(),
};

let result = scheduler.compile_round_robin(task).await?;
```

## Related Links

- [GCC Documentation](https://gcc.gnu.org/onlinedocs/)
- [Ollama Integration](ollama-en.md) - Local AI model support
- [Resource Manager](resource-manager-en.md) - Distributed resource management

## License

C Compiler Environment Management features follow to Vangriten AI Swarm license.
