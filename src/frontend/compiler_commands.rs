//! Tauri commands for the C compilation scheduler.

use crate::backend::BackendServices;
use crate::backend::c_compiler::{
    CGccInstance, CCompilationStatus, CCompilationResult, CCompilationTask,
};
use tauri::State;
use std::sync::Arc;
use std::path::PathBuf;

#[tauri::command]
pub async fn cmd_list_gcc_instances(
    state: State<'_, Arc<BackendServices>>,
) -> Result<Vec<CGccInstance>, String> {
    Ok(state.c_compiler.list_gcc_instances().await)
}

#[tauri::command]
pub async fn cmd_get_c_compiler_status(
    state: State<'_, Arc<BackendServices>>,
) -> Result<CCompilationStatus, String> {
    Ok(state.c_compiler.get_status().await)
}

#[tauri::command]
pub async fn cmd_compile_c_round_robin(
    source_files: Vec<String>,
    output_path: String,
    optimization_level: String,
    state: State<'_, Arc<BackendServices>>,
) -> Result<CCompilationResult, String> {
    let task = CCompilationTask {
        task_id: uuid::Uuid::new_v4().to_string(),
        source_files: source_files.into_iter().map(PathBuf::from).collect(),
        output_path: PathBuf::from(output_path),
        compiler_flags: vec!["-Wall".to_string(), "-Wextra".to_string()],
        include_paths: vec![],
        optimization_level,
    };

    state.c_compiler.compile_round_robin(task).await
        .map_err(|e| format!("Compilation failed: {:?}", e))
}

#[tauri::command]
pub async fn cmd_compile_c_parallel(
    tasks: Vec<CCompilationTask>,
    state: State<'_, Arc<BackendServices>>,
) -> Result<Vec<Result<CCompilationResult, String>>, String> {
    let results = state.c_compiler.compile_parallel(tasks).await;
    Ok(results.into_iter().map(|r| r.map_err(|e| format!("{:?}", e))).collect())
}
