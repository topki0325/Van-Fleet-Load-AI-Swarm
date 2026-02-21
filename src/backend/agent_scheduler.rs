use crate::shared::models::{
    Agent,
    AgentId,
    AgentStatus,
    AgentType,
    PerfMetrics,
    SkillVector,
    Task,
    TaskId,
    TaskOutput,
    TaskSpec,
    VgaError,
    SwarmPulse,
    TaskStatus,
};
use chrono::Utc;
use crate::backend::agents::{ArchitectAgent, EnvironmentAgent, ProgrammerAgent};
use crate::shared::models::AgentTrait;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::sync::atomic::Ordering;
use std::collections::HashMap;

pub struct AgentScheduler {
    agents: Arc<RwLock<Vec<Agent>>>,
    available_pool: Arc<RwLock<Vec<AgentId>>>,
    rotation_index: Arc<std::sync::atomic::AtomicUsize>,
    max_concurrency: usize,
    waiting_queue: Arc<RwLock<Vec<TaskId>>>,
    task_store: Arc<RwLock<HashMap<TaskId, Task>>>,
    active_tasks: Arc<RwLock<HashMap<TaskId, tokio::task::JoinHandle<Result<TaskOutput, VgaError>>>>>,
}

impl AgentScheduler {
    pub async fn new() -> Self {
        let scheduler = Self {
            agents: Arc::new(RwLock::new(vec![])),
            available_pool: Arc::new(RwLock::new(vec![])),
            rotation_index: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            max_concurrency: 100,
            waiting_queue: Arc::new(RwLock::new(vec![])),
            task_store: Arc::new(RwLock::new(HashMap::new())),
            active_tasks: Arc::new(RwLock::new(HashMap::new())),
        };

        // Seed a few default agents so the app has something to show.
        // This can be removed once agents are created via UI/commands.
        let defaults = vec![
            (AgentType::ArchitectNode, vec!["architecture".to_string(), "design".to_string()]),
            (AgentType::ProgrammerNode, vec!["rust".to_string(), "refactor".to_string()]),
            (AgentType::EnvManagerNode, vec!["env".to_string(), "build".to_string()]),
        ];
        for (role, skills) in defaults {
            let agent = Agent {
                id: uuid::Uuid::new_v4(),
                role,
                status: AgentStatus::Idle,
                skills: SkillVector { skills },
                current_task: None,
                performance: PerfMetrics {
                    cpu_usage: 0.0,
                    memory_usage: 0.0,
                    avg_response_time: std::time::Duration::from_millis(100),
                },
                heartbeat: Utc::now(),
            };
            let _ = scheduler.register_agent(agent).await;
        }

        scheduler
    }

    pub async fn register_agent(&self, agent: Agent) -> Result<(), VgaError> {
        let agent_id = agent.id;

        {
            let mut agents = self.agents.write().await;
            if agents.iter().any(|a| a.id == agent_id) {
                return Ok(());
            }
            agents.push(agent);
        }

        {
            let mut pool = self.available_pool.write().await;
            if !pool.contains(&agent_id) {
                pool.push(agent_id);
            }
        }

        Ok(())
    }

    pub async fn list_agents(&self) -> Vec<Agent> {
        self.agents.read().await.clone()
    }

    pub async fn execute_task_spec(&self, task_spec: TaskSpec) -> Result<TaskOutput, VgaError> {
        // Minimal routing logic:
        // - env tasks go to EnvironmentAgent
        // - known programming languages go to ProgrammerAgent
        // - otherwise fall back to ArchitectAgent
        let target = task_spec.target.to_lowercase();
        let language = task_spec.language.to_lowercase();

        if matches!(
            target.as_str(),
            "setup" | "health-check" | "allocate" | "env" | "environment"
        ) {
            let agent = EnvironmentAgent::new();
            return agent.execute_block(task_spec).await;
        }

        if matches!(
            language.as_str(),
            "rust" | "python" | "javascript" | "js" | "typescript" | "ts"
        ) {
            let agent = ProgrammerAgent::new();
            return agent.execute_block(task_spec).await;
        }

        let agent = ArchitectAgent::new();
        agent.execute_block(task_spec).await
    }

    pub fn gatling_rotate_next(&self) -> Result<Agent, VgaError> {
        let pool = self.available_pool.try_read().map_err(|_| VgaError::ResourceLimit("Lock poisoned".into()))?;
        if pool.is_empty() {
            return Err(VgaError::ResourceLimit("No agents available".into()));
        }

        let index = self.rotation_index.fetch_add(1, Ordering::SeqCst) % pool.len();
        let agent_id = pool[index];

        // Find the agent in the agents list
        let agents = self.agents.try_read().map_err(|_| VgaError::ResourceLimit("Lock poisoned".into()))?;
        agents.iter().find(|a| a.id == agent_id)
            .cloned()
            .ok_or_else(|| VgaError::AgentTimeout(agent_id))
    }

    pub async fn dispatch_task(&self, task: Task) -> Result<crate::shared::models::TaskHandle, VgaError> {
        let task_id = task.id;
        let agent = self.gatling_rotate_next()?;
        let handle = tokio::spawn(async move {
            agent.execute_block(task.spec).await
        });
        Ok(crate::shared::models::TaskHandle {
            task_id,
            handle,
        })
    }

    pub async fn get_swarm_status(&self) -> SwarmPulse {
        let pool_len = self.available_pool.read().await.len();
        let queue_len = self.waiting_queue.read().await.len();
        let active_count = self.active_tasks.read().await.len();
        SwarmPulse {
            total_agents: pool_len,
            active_tasks: active_count,
            queue_length: queue_len,
        }
    }

    pub fn handle_agent_heartbeat(&mut self, _agent_id: AgentId) {
        // TODO: Update agent's last heartbeat
    }

    pub async fn submit_task(&self, task: Task) -> Result<TaskId, VgaError> {
        let task_id = task.id;
        let mut task_store = self.task_store.write().await;
        task_store.insert(task_id, task);

        let mut queue = self.waiting_queue.write().await;
        queue.push(task_id);

        // Try to dispatch immediately if agents are available
        self.try_dispatch_next().await;

        Ok(task_id)
    }

    pub async fn get_task(&self, task_id: TaskId) -> Option<Task> {
        let task_store = self.task_store.read().await;
        task_store.get(&task_id).cloned()
    }

    pub async fn list_tasks(&self) -> Vec<Task> {
        let task_store = self.task_store.read().await;
        task_store.values().cloned().collect()
    }

    pub async fn cancel_task(&self, task_id: TaskId) -> Result<(), VgaError> {
        let mut task_store = self.task_store.write().await;
        if let Some(task) = task_store.get_mut(&task_id) {
            task.status = TaskStatus::Cancelled;
            task.updated_at = Utc::now();
        }

        // Remove from waiting queue if present
        let mut queue = self.waiting_queue.write().await;
        queue.retain(|&id| id != task_id);

        // Cancel active task if running
        let mut active_tasks = self.active_tasks.write().await;
        if let Some(handle) = active_tasks.remove(&task_id) {
            handle.abort();
        }

        Ok(())
    }

    async fn try_dispatch_next(&self) {
        let agent_available = {
            let pool = self.available_pool.read().await;
            !pool.is_empty()
        };

        if !agent_available {
            return;
        }

        let next_task_id = {
            let mut queue = self.waiting_queue.write().await;
            if queue.is_empty() {
                return;
            }
            queue.remove(0)
        };

        let task = {
            let task_store = self.task_store.read().await;
            match task_store.get(&next_task_id) {
                Some(task) if matches!(task.status, TaskStatus::Pending) => task.clone(),
                _ => return,
            }
        };

        // Mark task as running
        {
            let mut task_store = self.task_store.write().await;
            if let Some(task) = task_store.get_mut(&next_task_id) {
                task.status = TaskStatus::Running;
                task.updated_at = Utc::now();
            }
        }

        // Dispatch to agent
        if let Ok(agent) = self.gatling_rotate_next() {
            let task_store = self.task_store.clone();
            let active_tasks = self.active_tasks.clone();

            let handle = tokio::spawn(async move {
                let result = agent.execute_block(task.spec).await;
                {
                    let mut task_store = task_store.write().await;
                    if let Some(task) = task_store.get_mut(&next_task_id) {
                        task.status = match &result {
                            Ok(_) => TaskStatus::Completed,
                            Err(_) => TaskStatus::Failed,
                        };
                        task.output = match &result {
                            Ok(output) => crate::shared::models::TaskResult::Success(output.clone()),
                            Err(e) => crate::shared::models::TaskResult::Failure(format!("{:?}", e)),
                        };
                        task.updated_at = Utc::now();
                    }
                }
                {
                    let mut active_tasks = active_tasks.write().await;
                    active_tasks.remove(&next_task_id);
                }
                result
            });

            let mut active_tasks = self.active_tasks.write().await;
            active_tasks.insert(next_task_id, handle);
        }
    }

    pub async fn process_completed_tasks(&self) {
        // Clean up completed tasks and try to dispatch more
        self.try_dispatch_next().await;
    }
}