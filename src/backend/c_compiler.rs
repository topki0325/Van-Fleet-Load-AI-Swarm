use crate::shared::models::{BuildOutput, BuildPlan, VgaError};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::Arc;
use tokio::sync::{RwLock, Semaphore};
use tokio::process::Command as TokioCommand;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CGccInstance {
    pub id: String,
    pub gcc_path: PathBuf,
    pub is_available: bool,
    pub current_task: Option<String>,
}

#[derive(Clone)]
pub struct CCompilationScheduler {
    gcc_instances: Arc<RwLock<Vec<CGccInstance>>>,
    max_parallel_jobs: usize,
    current_index: Arc<RwLock<usize>>,
    semaphore: Arc<Semaphore>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CCompilationTask {
    pub task_id: String,
    pub source_files: Vec<PathBuf>,
    pub output_path: PathBuf,
    pub compiler_flags: Vec<String>,
    pub include_paths: Vec<PathBuf>,
    pub optimization_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CCompilationResult {
    pub task_id: String,
    pub success: bool,
    pub binary_path: Option<PathBuf>,
    pub output: String,
    pub error_output: String,
    pub compilation_time_ms: u64,
    pub gcc_instance_id: String,
}

impl CCompilationScheduler {
    pub async fn new(max_parallel_jobs: usize) -> Result<Self, VgaError> {
        let gcc_instances = Self::discover_gcc_instances().await?;

        if gcc_instances.is_empty() {
            tracing::warn!("No GCC instances found; C compiler scheduler will be disabled");
        }

        let permits = if gcc_instances.is_empty() {
            0
        } else {
            max_parallel_jobs.min(gcc_instances.len())
        };
        let semaphore = Arc::new(Semaphore::new(permits));

        Ok(Self {
            gcc_instances: Arc::new(RwLock::new(gcc_instances)),
            max_parallel_jobs,
            current_index: Arc::new(RwLock::new(0)),
            semaphore,
        })
    }

    async fn discover_gcc_instances() -> Result<Vec<CGccInstance>, VgaError> {
        let mut instances = Vec::new();
        
        let gcc_paths = vec![
            "gcc",
            "gcc-12",
            "gcc-11",
            "gcc-10",
            "cc",
            "/usr/bin/gcc",
            "/usr/local/bin/gcc",
        ];

        for (i, gcc_path) in gcc_paths.iter().enumerate() {
            if let Ok(output) = Command::new(gcc_path)
                .arg("--version")
                .output()
            {
                if output.status.success() {
                    let version = String::from_utf8_lossy(&output.stdout)
                        .lines()
                        .next()
                        .unwrap_or("unknown")
                        .to_string();
                    
                    instances.push(CGccInstance {
                        id: format!("gcc-{}", i),
                        gcc_path: PathBuf::from(gcc_path),
                        is_available: true,
                        current_task: None,
                    });
                    
                    let resolved = PathBuf::from(gcc_path)
                        .canonicalize()
                        .unwrap_or_else(|_| PathBuf::from(gcc_path));
                    tracing::info!(
                        "Found GCC instance: {} at {} ({})",
                        gcc_path,
                        resolved.display(),
                        version
                    );
                }
            }
        }

        if instances.is_empty() {
            tracing::warn!("No GCC instances found in system");
        }

        Ok(instances)
    }

    pub async fn list_gcc_instances(&self) -> Vec<CGccInstance> {
        self.gcc_instances.read().await.clone()
    }

    pub async fn compile_round_robin(&self, task: CCompilationTask) -> Result<CCompilationResult, VgaError> {
        let instances = self.gcc_instances.read().await;
        let available_instances: Vec<CGccInstance> = instances
            .iter()
            .filter(|inst| inst.is_available && inst.current_task.is_none())
            .cloned()
            .collect();

        if available_instances.is_empty() {
            return Err(VgaError::ResourceLimit("No available GCC instances".into()));
        }

        let mut index = self.current_index.write().await;
        let selected = available_instances[*index % available_instances.len()].clone();
        *index = (*index + 1) % available_instances.len();
        drop(index);

        self.compile_with_instance(selected.clone(), task).await
    }

    pub async fn compile_parallel(&self, tasks: Vec<CCompilationTask>) -> Vec<Result<CCompilationResult, VgaError>> {
        let mut results = Vec::new();
        let mut handles = Vec::new();

        for task in tasks {
            let instances = self.gcc_instances.clone();
            let semaphore = self.semaphore.clone();
            
            let handle = tokio::spawn(async move {
                let _permit = semaphore.acquire().await;
                
                let instances_guard = instances.read().await;
                let available: Vec<CGccInstance> = instances_guard
                    .iter()
                    .filter(|inst| inst.is_available && inst.current_task.is_none())
                    .cloned()
                    .collect();
                
                if let Some(instance) = available.first() {
                    Self::compile_with_instance_internal(instance.clone(), task).await
                } else {
                    Err(VgaError::ResourceLimit("No available GCC instances".into()))
                }
            });
            
            handles.push(handle);
        }

        for handle in handles {
            results.push(handle.await.unwrap_or_else(|e| {
                Err(VgaError::ResourceLimit(format!("Task failed: {:?}", e)))
            }));
        }

        results
    }

    async fn compile_with_instance(&self, instance: CGccInstance, task: CCompilationTask) -> Result<CCompilationResult, VgaError> {
        let instance_id = instance.id.clone();
        let mut instances = self.gcc_instances.write().await;
        let instance_ref = instances.iter_mut()
            .find(|inst| inst.id == instance.id)
            .ok_or_else(|| VgaError::ResourceLimit("Instance not found".into()))?;
        
        instance_ref.current_task = Some(task.task_id.clone());
        drop(instances);

        let result = Self::compile_with_instance_internal(instance, task).await;

        let mut instances = self.gcc_instances.write().await;
        if let Some(inst) = instances.iter_mut().find(|inst| inst.id == instance_id) {
            inst.current_task = None;
        }

        result
    }

    async fn compile_with_instance_internal(instance: CGccInstance, task: CCompilationTask) -> Result<CCompilationResult, VgaError> {
        let start = std::time::Instant::now();

        let output_dir = task.output_path.parent()
            .ok_or_else(|| VgaError::ResourceLimit("Invalid output path".into()))?;

        tokio::fs::create_dir_all(output_dir).await
            .map_err(|e| VgaError::ResourceLimit(format!("Failed to create output directory: {}", e)))?;

        let mut cmd = TokioCommand::new(&instance.gcc_path);
        
        for file in &task.source_files {
            cmd.arg(file);
        }

        cmd.arg("-o")
            .arg(&task.output_path);

        cmd.arg(format!("-O{}", task.optimization_level));

        for flag in &task.compiler_flags {
            cmd.arg(flag);
        }

        for include_path in &task.include_paths {
            cmd.arg(format!("-I{}", include_path.display()));
        }

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        tracing::info!("Compiling with GCC instance {}: {:?}", instance.id, cmd);

        let child = cmd.spawn()
            .map_err(|e| VgaError::ResourceLimit(format!("Failed to spawn GCC: {}", e)))?;

        let output = child.wait_with_output().await
            .map_err(|e| VgaError::ResourceLimit(format!("Failed to wait for GCC: {}", e)))?;

        let compilation_time_ms = start.elapsed().as_millis() as u64;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let success = output.status.success();

        if success {
            tracing::info!("Compilation successful: {} in {}ms", task.task_id, compilation_time_ms);
        } else {
            tracing::error!("Compilation failed: {} - {}", task.task_id, stderr);
        }

        Ok(CCompilationResult {
            task_id: task.task_id,
            success,
            binary_path: if success { Some(task.output_path.clone()) } else { None },
            output: stdout,
            error_output: stderr,
            compilation_time_ms,
            gcc_instance_id: instance.id,
        })
    }

    #[allow(dead_code)]
    pub async fn compile_project(&self, plan: BuildPlan) -> Result<Vec<BuildOutput>, VgaError> {
        let mut outputs = Vec::new();

        for segment in &plan.segments {
            if segment.language.to_lowercase() != "c" {
                continue;
            }

            let task = CCompilationTask {
                task_id: Uuid::new_v4().to_string(),
                source_files: segment.files.clone(),
                output_path: PathBuf::from(format!("target/{}.out", segment.files.first()
                    .and_then(|p| p.file_stem())
                    .unwrap_or_else(|| std::ffi::OsStr::new("output"))
                    .to_string_lossy())),
                compiler_flags: vec!["-Wall".to_string(), "-Wextra".to_string()],
                include_paths: vec![],
                optimization_level: "2".to_string(),
            };

            let result = self.compile_round_robin(task).await?;

            outputs.push(BuildOutput {
                segment_id: Uuid::new_v4().to_string(),
                binary_path: result.binary_path.unwrap_or_else(|| PathBuf::from("")),
                success: result.success,
                error_message: if result.success { None } else { Some(result.error_output) },
            });
        }

        Ok(outputs)
    }

    #[allow(dead_code)]
    pub async fn compile_project_parallel(&self, plan: BuildPlan) -> Result<Vec<BuildOutput>, VgaError> {
        let mut tasks = Vec::new();
        let mut outputs = Vec::new();

        for segment in &plan.segments {
            if segment.language.to_lowercase() != "c" {
                continue;
            }

            let task = CCompilationTask {
                task_id: Uuid::new_v4().to_string(),
                source_files: segment.files.clone(),
                output_path: PathBuf::from(format!("target/{}.out", segment.files.first()
                    .and_then(|p| p.file_stem())
                    .unwrap_or_else(|| std::ffi::OsStr::new("output"))
                    .to_string_lossy())),
                compiler_flags: vec!["-Wall".to_string(), "-Wextra".to_string()],
                include_paths: vec![],
                optimization_level: "2".to_string(),
            };

            tasks.push(task);
        }

        let results = self.compile_parallel(tasks).await;

        for result in results {
            match result {
                Ok(comp_result) => {
                    outputs.push(BuildOutput {
                        segment_id: Uuid::new_v4().to_string(),
                        binary_path: comp_result.binary_path.unwrap_or_else(|| PathBuf::from("")),
                        success: comp_result.success,
                        error_message: if comp_result.success { None } else { Some(comp_result.error_output) },
                    });
                }
                Err(e) => {
                    outputs.push(BuildOutput {
                        segment_id: Uuid::new_v4().to_string(),
                        binary_path: PathBuf::from(""),
                        success: false,
                        error_message: Some(format!("Compilation error: {:?}", e)),
                    });
                }
            }
        }

        Ok(outputs)
    }

    pub async fn get_status(&self) -> CCompilationStatus {
        let instances = self.gcc_instances.read().await;
        let available_count = instances.iter()
            .filter(|inst| inst.is_available && inst.current_task.is_none())
            .count();
        let busy_count = instances.iter()
            .filter(|inst| inst.current_task.is_some())
            .count();

        CCompilationStatus {
            total_instances: instances.len(),
            available_instances: available_count,
            busy_instances: busy_count,
            max_parallel_jobs: self.max_parallel_jobs,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CCompilationStatus {
    pub total_instances: usize,
    pub available_instances: usize,
    pub busy_instances: usize,
    pub max_parallel_jobs: usize,
}