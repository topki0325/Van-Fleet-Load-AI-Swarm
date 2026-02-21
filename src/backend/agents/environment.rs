use crate::shared::models::{AgentTrait, TaskOutput, VgaError, TaskSpec, ContextManager, PerfMetrics, EnvSpec};

#[derive(Clone)]
pub struct EnvironmentAgent {
    context: ContextManager,
    active_environments: std::collections::HashMap<String, EnvironmentStatus>,
}

#[derive(Clone, Debug)]
pub enum EnvironmentStatus {
    Available,
    InUse,
    Maintenance,
    Failed,
}

impl EnvironmentAgent {
    pub fn new() -> Self {
        Self {
            context: ContextManager {
                memory_slots: std::collections::HashMap::new(),
                docs: vec![],
            },
            active_environments: std::collections::HashMap::new(),
        }
    }

    /// Setup environment based on specifications
    fn setup_environment(&mut self, env_spec: &EnvSpec) -> Result<String, VgaError> {
        let env_id = format!("env_{}_{}", env_spec.language, env_spec.version);

        // Check if environment already exists
        if self.active_environments.contains_key(&env_id) {
            return Ok(format!("Environment {} already exists and is ready", env_id));
        }

        // Create environment setup script
        let setup_script = self.generate_setup_script(env_spec)?;

        // Mark environment as available
        self.active_environments.insert(env_id.clone(), EnvironmentStatus::Available);

        Ok(format!("Environment {} setup complete.\n\nSetup Script:\n{}", env_id, setup_script))
    }

    /// Generate setup script for the environment
    fn generate_setup_script(&self, env_spec: &EnvSpec) -> Result<String, VgaError> {
        match env_spec.language.to_lowercase().as_str() {
            "rust" => self.generate_rust_setup(&env_spec),
            "python" => self.generate_python_setup(&env_spec),
            "node" | "javascript" | "typescript" => self.generate_node_setup(&env_spec),
            _ => Err(VgaError::CompileFailure(format!("Unsupported language: {}", env_spec.language))),
        }
    }

    fn generate_rust_setup(&self, env_spec: &EnvSpec) -> Result<String, VgaError> {
        let mut script = format!(
            "# Rust Environment Setup for version {}\n",
            env_spec.version
        );

        script.push_str("# Install Rust if not present\n");
        script.push_str("curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y\n");
        script.push_str("source $HOME/.cargo/env\n");
        script.push_str("\n# Verify installation\n");
        script.push_str("rustc --version\n");
        script.push_str("cargo --version\n");

        if !env_spec.dependencies.is_empty() {
            script.push_str("\n# Install dependencies\n");
            for dep in &env_spec.dependencies {
                script.push_str(&format!("cargo add {}\n", dep));
            }
        }

        Ok(script)
    }

    fn generate_python_setup(&self, env_spec: &EnvSpec) -> Result<String, VgaError> {
        let mut script = format!(
            "# Python Environment Setup for version {}\n",
            env_spec.version
        );

        script.push_str("# Create virtual environment\n");
        script.push_str(&format!("python{} -m venv venv\n", env_spec.version));
        script.push_str("source venv/bin/activate  # On Windows: venv\\Scripts\\activate\n");
        script.push_str("\n# Upgrade pip\n");
        script.push_str("pip install --upgrade pip\n");

        if !env_spec.dependencies.is_empty() {
            script.push_str("\n# Install dependencies\n");
            for dep in &env_spec.dependencies {
                script.push_str(&format!("pip install {}\n", dep));
            }
        }

        script.push_str("\n# Verify installation\n");
        script.push_str("python --version\n");
        script.push_str("pip list\n");

        Ok(script)
    }

    fn generate_node_setup(&self, env_spec: &EnvSpec) -> Result<String, VgaError> {
        let mut script = format!(
            "# Node.js Environment Setup for version {}\n",
            env_spec.version
        );

        script.push_str("# Install Node.js using nvm (recommended)\n");
        script.push_str("curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash\n");
        script.push_str("source ~/.bashrc\n");
        script.push_str(&format!("nvm install {}\n", env_spec.version));
        script.push_str(&format!("nvm use {}\n", env_spec.version));
        script.push_str("\n# Verify installation\n");
        script.push_str("node --version\n");
        script.push_str("npm --version\n");

        if !env_spec.dependencies.is_empty() {
            script.push_str("\n# Install dependencies\n");
            script.push_str("npm init -y\n");
            for dep in &env_spec.dependencies {
                script.push_str(&format!("npm install {}\n", dep));
            }
        }

        Ok(script)
    }

    /// Check environment health
    fn check_environment_health(&self, env_id: &str) -> Result<String, VgaError> {
        let status = self.active_environments.get(env_id)
            .ok_or_else(|| VgaError::CompileFailure(format!("Environment {} not found", env_id)))?;

        match status {
            EnvironmentStatus::Available => Ok(format!("Environment {} is healthy and available", env_id)),
            EnvironmentStatus::InUse => Ok(format!("Environment {} is currently in use", env_id)),
            EnvironmentStatus::Maintenance => Ok(format!("Environment {} is under maintenance", env_id)),
            EnvironmentStatus::Failed => Err(VgaError::CompileFailure(format!("Environment {} has failed", env_id))),
        }
    }

    /// Allocate resources for a task
    fn allocate_resources(&mut self, task_id: &str, requirements: &str) -> Result<String, VgaError> {
        // Simple resource allocation logic
        let resources = self.parse_resource_requirements(requirements)?;

        // Find suitable environment
        let suitable_env = self.find_suitable_environment(&resources)?;

        // Mark environment as in use
        if let Some(status) = self.active_environments.get_mut(&suitable_env) {
            *status = EnvironmentStatus::InUse;
        }

        Ok(format!(
            "Allocated resources for task {}:\n- Environment: {}\n- CPU: {} cores\n- Memory: {} MB\n- Disk: {} GB",
            task_id, suitable_env, resources.cpu_cores, resources.memory_mb, resources.disk_gb
        ))
    }

    fn parse_resource_requirements(&self, requirements: &str) -> Result<ResourceRequirements, VgaError> {
        // Simple parsing - in real implementation, this would be more sophisticated
        let cpu = if requirements.contains("high-cpu") { 4 } else { 2 };
        let memory = if requirements.contains("high-memory") { 4096 } else { 2048 };
        let disk = if requirements.contains("large-disk") { 50 } else { 10 };

        Ok(ResourceRequirements {
            cpu_cores: cpu,
            memory_mb: memory,
            disk_gb: disk,
        })
    }

    fn find_suitable_environment(&mut self, _resources: &ResourceRequirements) -> Result<String, VgaError> {
        for (env_id, status) in &self.active_environments {
            if matches!(status, EnvironmentStatus::Available) {
                return Ok(env_id.clone());
            }
        }

        let new_env = format!("env_dynamic_{}", self.active_environments.len());
        self.active_environments
            .insert(new_env.clone(), EnvironmentStatus::Available);
        Ok(new_env)
    }

    fn parse_environment_instruction(&self, instruction: &str) -> Result<EnvSpec, VgaError> {
        // Simple parsing logic - in real implementation, this would be more sophisticated
        let instr_lower = instruction.to_lowercase();

        let language = if instr_lower.contains("rust") {
            "rust".to_string()
        } else if instr_lower.contains("python") {
            "python".to_string()
        } else if instr_lower.contains("node") || instr_lower.contains("javascript") {
            "node".to_string()
        } else {
            "rust".to_string() // Default
        };

        let version = if instr_lower.contains("1.7") {
            "1.70".to_string()
        } else if instr_lower.contains("3.1") {
            "3.11".to_string()
        } else if instr_lower.contains("18") {
            "18".to_string()
        } else {
            "latest".to_string()
        };

        // Extract dependencies from instruction
        let dependencies = self.extract_dependencies(&instr_lower);

        Ok(EnvSpec {
            language,
            version,
            dependencies,
        })
    }

    fn extract_dependencies(&self, instruction: &str) -> Vec<String> {
        let mut deps = Vec::new();

        if instruction.contains("serde") {
            deps.push("serde".to_string());
            deps.push("serde_json".to_string());
        }
        if instruction.contains("tokio") {
            deps.push("tokio".to_string());
        }
        if instruction.contains("warp") || instruction.contains("web") {
            deps.push("warp".to_string());
        }
        if instruction.contains("reqwest") || instruction.contains("http") {
            deps.push("reqwest".to_string());
        }

        deps
    }
}

#[derive(Debug)]
struct ResourceRequirements {
    cpu_cores: u32,
    memory_mb: u32,
    disk_gb: u32,
}

#[async_trait::async_trait]
impl AgentTrait for EnvironmentAgent {
    async fn execute_instruction(&self, instr: String) -> Result<TaskOutput, VgaError> {
        // Parse instruction and setup environment
        let env_spec = self.parse_environment_instruction(&instr)?;
        let mut agent = self.clone();
        let result = agent.setup_environment(&env_spec)?;

        Ok(TaskOutput {
            content: result,
            metadata: {
                let mut meta = std::collections::HashMap::new();
                meta.insert("language".to_string(), env_spec.language);
                meta.insert("version".to_string(), env_spec.version);
                meta.insert("dependencies_count".to_string(), env_spec.dependencies.len().to_string());
                meta
            },
        })
    }

    async fn execute_block(&self, task_spec: TaskSpec) -> Result<TaskOutput, VgaError> {
        match task_spec.target.as_str() {
            "setup" => {
                let env_spec = EnvSpec {
                    language: task_spec.language.clone(),
                    version: "latest".to_string(), // Default version
                    dependencies: vec![], // Parse from context
                };
                let mut agent = self.clone();
                let result = agent.setup_environment(&env_spec)?;
                Ok(TaskOutput {
                    content: result,
                    metadata: std::collections::HashMap::new(),
                })
            },
            "health-check" => {
                let env_id = format!("env_{}_latest", task_spec.language);
                let result = self.check_environment_health(&env_id)?;
                Ok(TaskOutput {
                    content: result,
                    metadata: std::collections::HashMap::new(),
                })
            },
            "allocate" => {
                let mut agent = self.clone();
                let result = agent.allocate_resources("task_unknown", &task_spec.context_range)?;
                Ok(TaskOutput {
                    content: result,
                    metadata: std::collections::HashMap::new(),
                })
            },
            _ => Err(VgaError::CompileFailure(format!("Unknown environment task: {}", task_spec.target))),
        }
    }

    fn update_context(&mut self, context: &ContextManager) {
        self.context = context.clone();
    }

    fn get_metrics(&self) -> PerfMetrics {
        PerfMetrics {
            cpu_usage: 0.0,
            memory_usage: 0.0,
            avg_response_time: std::time::Duration::from_millis(200),
        }
    }
}