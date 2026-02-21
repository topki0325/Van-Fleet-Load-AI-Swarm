# C语言编译环境管理

## 概述

Vangriten AI Swarm 提供了强大的C语言编译环境管理功能，支持多个GCC实例的轮流和并行编译。系统会自动发现系统中的GCC编译器，并提供智能的编译调度。

## 核心功能

### 1. GCC实例发现

系统会自动搜索并发现系统中的GCC编译器实例：
- `gcc`
- `gcc-12`, `gcc-11`, `gcc-10`
- `cc`
- `/usr/bin/gcc`
- `/usr/local/bin/gcc`

每个实例都会被记录其路径、版本和可用性状态。

### 2. 轮流编译策略 (Round Robin)

轮流编译策略会在多个可用的GCC实例之间轮流分配编译任务，实现负载均衡：

```rust
let result = scheduler.compile_round_robin(task).await?;
```

**优点：**
- 简单易实现
- 负载均衡
- 无需额外配置

**适用场景：**
- 小型项目
- 编译任务数量适中
- 各GCC实例性能相近

### 3. 并行编译策略 (Parallel)

并行编译策略可以同时使用多个GCC实例编译多个文件：

```rust
let results = scheduler.compile_parallel(tasks).await;
```

**优点：**
- 最大化利用多核CPU
- 显著减少编译时间
- 适合大型项目

**适用场景：**
- 大型项目
- 多个独立源文件
- 需要快速编译

### 4. 编译任务配置

每个编译任务可以配置以下参数：

```rust
pub struct CCompilationTask {
    pub task_id: String,              // 任务唯一标识
    pub source_files: Vec<PathBuf>,   // 源文件列表
    pub output_path: PathBuf,         // 输出文件路径
    pub compiler_flags: Vec<String>,    // 编译器标志
    pub include_paths: Vec<PathBuf>,    // 包含路径
    pub optimization_level: String,      // 优化级别 (0, 1, 2, 3, s)
}
```

**优化级别说明：**
- `O0` - 无优化，编译速度快
- `O1` - 基本优化
- `O2` - 推荐的优化级别（默认）
- `O3` - 激进优化，编译时间较长
- `Os` - 代码大小优化

### 5. 并发控制

系统使用信号量（Semaphore）控制最大并行编译任务数：

```rust
let scheduler = CCompilationScheduler::new(4).await?;
```

这确保不会同时运行超过4个编译任务，避免系统过载。

## 使用方法

### 通过Web界面使用

#### 1. 查看GCC实例

点击 "List GCC Instances" 按钮查看所有发现的GCC实例：

```
GCC Instances:
  - gcc-0 at /usr/bin/gcc (gcc (Ubuntu 11.4.0-1ubuntu1~22.04) 11.4.0)
    Available: true
    Current task: None
```

#### 2. 查看编译器状态

点击 "Get Status" 按钮查看当前编译器状态：

```
Compiler Status:
  Total instances: 1
  Available instances: 1
  Busy instances: 0
  Max parallel jobs: 4
```

#### 3. 轮流编译

在 "C Compilation (Round Robin)" 部分：
1. 输入源文件（逗号分隔）
2. 输入输出路径
3. 选择优化级别
4. 点击 "Compile (Round Robin)" 按钮

示例：
```
Source Files: main.c,utils.c
Output Path: target/myapp
Optimization Level: O2 - Recommended
```

#### 4. 并行编译

在 "C Compilation (Parallel)" 部分：
1. 输入JSON格式的任务数组
2. 点击 "Compile (Parallel)" 按钮

示例：
```json
[
  {
    "task_id": "task1",
    "source_files": ["main.c"],
    "output_path": "target/main",
    "compiler_flags": ["-Wall", "-Wextra"],
    "include_paths": ["include"],
    "optimization_level": "2"
  },
  {
    "task_id": "task2",
    "source_files": ["utils.c"],
    "output_path": "target/utils",
    "compiler_flags": ["-Wall"],
    "include_paths": ["include"],
    "optimization_level": "2"
  }
]
```

### 通过代码调用

#### 初始化编译调度器

```rust
use vangriten_ai_swarm::backend::CCompilationScheduler;

let scheduler = CCompilationScheduler::new(4).await?;
```

#### 轮流编译单个任务

```rust
use vangriten_ai_swarm::backend::CCompilationTask;

let task = CCompilationTask {
    task_id: uuid::Uuid::new_v4().to_string(),
    source_files: vec![std::path::PathBuf::from("main.c")],
    output_path: std::path::PathBuf::from("target/main"),
    compiler_flags: vec!["-Wall".to_string(), "-Wextra".to_string()],
    include_paths: vec![],
    optimization_level: "2".to_string(),
};

let result = scheduler.compile_round_robin(task).await?;

if result.success {
    println!("Compilation successful: {:?}", result.binary_path);
} else {
    println!("Compilation failed: {}", result.error_output);
}
```

#### 并行编译多个任务

```rust
let tasks = vec![
    CCompilationTask {
        task_id: "task1".to_string(),
        source_files: vec![std::path::PathBuf::from("main.c")],
        output_path: std::path::PathBuf::from("target/main"),
        compiler_flags: vec!["-Wall".to_string()],
        include_paths: vec![],
        optimization_level: "2".to_string(),
    },
    CCompilationTask {
        task_id: "task2".to_string(),
        source_files: vec![std::path::PathBuf::from("utils.c")],
        output_path: std::path::PathBuf::from("target/utils"),
        compiler_flags: vec!["-Wall".to_string()],
        include_paths: vec![],
        optimization_level: "2".to_string(),
    },
];

let results = scheduler.compile_parallel(tasks).await;

for (i, result) in results.into_iter().enumerate() {
    match result {
        Ok(comp_result) => {
            println!("Task {} compiled in {:?}", i, comp_result.compilation_time);
            if comp_result.success {
                println!("  Output: {:?}", comp_result.binary_path);
            } else {
                println!("  Error: {}", comp_result.error_output);
            }
        }
        Err(e) => {
            println!("Task {} failed: {:?}", i, e);
        }
    }
}
```

#### 查看编译器状态

```rust
let status = scheduler.get_status().await;
println!("Total instances: {}", status.total_instances);
println!("Available: {}", status.available_instances);
println!("Busy: {}", status.busy_instances);
println!("Max parallel jobs: {}", status.max_parallel_jobs);
```

## 编译结果

每个编译任务都会返回详细的结果：

```rust
pub struct CCompilationResult {
    pub task_id: String,              // 任务ID
    pub success: bool,               // 是否成功
    pub binary_path: Option<PathBuf>, // 输出二进制文件路径
    pub output: String,               // 编译器输出
    pub error_output: String,          // 错误输出
    pub compilation_time: Duration,      // 编译耗时
    pub gcc_instance_id: String,       // 使用的GCC实例ID
}
```

## 示例项目

项目包含一个简单的C语言示例程序：

```c
#include <stdio.h>

int main() {
    printf("Hello from Vangriten AI Swarm C Compiler!\n");
    return 0;
}
```

编译命令：
```bash
# 通过Web界面
Source Files: examples/hello.c
Output Path: target/hello
Optimization Level: O2 - Recommended

# 或通过命令行
cargo run --bin resource_manager
resource-manager> request
# 然后使用编译功能
```

## 性能优化建议

### 1. 选择合适的优化级别

- **开发阶段**: 使用 `O0` 或 `O1`，编译速度快
- **测试阶段**: 使用 `O2`，平衡编译时间和性能
- **生产阶段**: 使用 `O3` 或 `Os`，最大化性能

### 2. 并行编译策略

对于大型项目，使用并行编译可以显著减少编译时间：

```rust
let tasks = project_files.into_iter()
    .map(|file| create_compilation_task(file))
    .collect();

let results = scheduler.compile_parallel(tasks).await;
```

### 3. 合理设置并发数

根据CPU核心数设置最大并行任务数：

```rust
// 获取CPU核心数
let cpu_cores = num_cpus::get();

// 设置为CPU核心数或稍少
let max_parallel = cpu_cores - 1;

let scheduler = CCompilationScheduler::new(max_parallel).await?;
```

### 4. 使用增量编译

只重新编译修改的文件，减少不必要的编译：

```rust
let modified_files = get_modified_files();
let tasks: Vec<_> = modified_files.into_iter()
    .filter(|file| needs_recompilation(file))
    .map(|file| create_task(file))
    .collect();

let results = scheduler.compile_parallel(tasks).await;
```

## 故障排除

### GCC未找到

**问题**: 系统提示 "No GCC instances found"

**解决方案**:
1. 安装GCC: `sudo apt-get install gcc` (Ubuntu/Debian)
2. 检查PATH环境变量
3. 确认GCC路径正确

### 编译失败

**问题**: 编译返回错误

**解决方案**:
1. 查看错误输出: `result.error_output`
2. 检查源文件语法
3. 验证包含路径正确
4. 检查编译器标志是否正确

### 并发限制

**问题**: 并行编译时系统变慢

**解决方案**:
1. 减少最大并行任务数
2. 检查系统资源使用情况
3. 关闭其他占用CPU的程序

## 最佳实践

1. **模块化代码**: 将代码分成多个独立模块，便于并行编译
2. **合理组织文件**: 按功能组织源文件，减少依赖
3. **使用头文件**: 将公共声明放在头文件中，减少重复编译
4. **清理输出**: 定期清理编译输出目录，避免混淆
5. **监控性能**: 跟踪编译时间，优化编译策略

## 扩展功能

### 添加自定义编译器

如果需要使用其他编译器（如clang），可以扩展 `discover_gcc_instances` 方法：

```rust
async fn discover_gcc_instances() -> Result<Vec<CGccInstance>, VgaError> {
    let mut instances = Vec::new();
    
    let compiler_paths = vec![
        "gcc",
        "clang",
        "cc",
    ];

    for (i, compiler_path) in compiler_paths.iter().enumerate() {
        if let Ok(output) = Command::new(compiler_path)
            .arg("--version")
            .output() {
            // ... 添加实例
        }
    }

    Ok(instances)
}
```

### 自定义编译标志

根据项目需求添加自定义编译标志：

```rust
let task = CCompilationTask {
    compiler_flags: vec![
        "-Wall".to_string(),
        "-Wextra".to_string(),
        "-O2".to_string(),
        "-march=native".to_string(),  // 针对本地CPU优化
        "-flto".to_string(),            // 链接时优化
    ],
    // ...
};
```

## 相关文档

- [资源管理代理](resource-manager.md)
- [API密钥管理](api-key-management.md)

## 许可证

MIT License