use crate::app::VgaGuiApp;
use crate::app_types::ProviderFilter;
use vangriten_ai_swarm::shared::models::*;

impl VgaGuiApp {
    pub fn refresh_all(&mut self) {
        self.clear_error();
        let services = self.services.clone();

        let result = self.runtime.block_on(async move {
            let swarm = services.agent_scheduler.get_swarm_status().await;
            let agents = services.agent_scheduler.list_agents().await;
            let projects = services.projects.read().await.clone();
            let leases = services.leases.read().await.clone();
            let tasks = services.agent_scheduler.list_tasks().await;
            (swarm, agents, projects, leases, tasks)
        });

        self.swarm_json = Self::pretty(&result.0);
        self.agents_json = Self::pretty(&result.1);
        self.projects_json = Self::pretty(&result.2);
        self.leases_json = Self::pretty(&result.3);
        self.tasks_json = Self::pretty(&result.4);

        self.last_refresh_instant = std::time::Instant::now();
    }

    pub fn deploy_sample_project(&mut self) {
        self.clear_error();
        let services = self.services.clone();

        let result = self.runtime.block_on(async move {
            let project_id = uuid::Uuid::new_v4();
            let project = Project {
                id: project_id,
                name: format!("project-{project_id}"),
                config: ProjectConfig {
                    tech_stack: vec!["rust".to_string(), "tauri".to_string()],
                    default_provider: "local".to_string(),
                    concurrency_strategy: "gatling".to_string(),
                },
                agents: Vec::new(),
                workflow: WorkflowGraph::default(),
                state: ProjectStatus::Initialized,
                stats: ExecutionStats {
                    total_tokens: 0,
                    total_duration: std::time::Duration::from_secs(0),
                    total_cost: 0.0,
                },
                last_updated: chrono::Utc::now(),
            };

            services.projects.write().await.push(project);
            project_id
        });

        self.projects_json = format!("Deployed project: {result}\n\n{}", self.projects_json);
        self.refresh_all();
    }

    pub fn request_sample_compute(&mut self) {
        self.clear_error();
        let services = self.services.clone();

        self.runtime.block_on(async move {
            let lease = ResourceLease {
                id: uuid::Uuid::new_v4().to_string(),
                gpu_memory: 0,
                duration: std::time::Duration::from_secs(600),
            };
            services.leases.write().await.push(lease);
        });

        self.refresh_all();
    }

    pub fn execute_task(&mut self) {
        self.clear_error();
        let services = self.services.clone();
        let spec = TaskSpec {
            language: self.task_language.clone(),
            target: self.task_target.clone(),
            context_range: self.task_context.clone(),
        };

        let res = self.runtime.block_on(async move {
            services.agent_scheduler.execute_task_spec(spec).await
        });

        match res {
            Ok(out) => self.tasks_json = format!("Last execute_task output:\n{}\n\n{}", Self::pretty(&out), self.tasks_json),
            Err(e) => self.set_error(format!("execute_task failed: {e:?}")),
        }

        self.refresh_all();
    }

    pub fn submit_task(&mut self) {
        self.clear_error();
        let services = self.services.clone();
        let task = Task::new(
            TaskSpec {
                language: self.task_language.clone(),
                target: self.task_target.clone(),
                context_range: self.task_context.clone(),
            },
            Priority::Medium,
            std::path::PathBuf::from("snapshots/gui.json"),
        );

        let res = self.runtime.block_on(async move { services.agent_scheduler.submit_task(task).await });
        match res {
            Ok(id) => self.tasks_json = format!("Submitted task: {id}\n\n{}", self.tasks_json),
            Err(e) => self.set_error(format!("submit_task failed: {e:?}")),
        }

        self.refresh_all();
    }

    pub fn load_providers(&mut self) {
        self.clear_error();

        let res = self
            .services
            .api_manager
            .vault_operation(VaultOp::GetProviders);

        match res {
            Ok(VaultResult::ProviderConfigs(list)) => {
                let filtered: Vec<_> = match self.provider_filter {
                    ProviderFilter::All => list,
                    ProviderFilter::China => list
                        .into_iter()
                        .filter(|p| p.region == ProviderRegion::China)
                        .collect(),
                    ProviderFilter::USA => list
                        .into_iter()
                        .filter(|p| p.region == ProviderRegion::USA)
                        .collect(),
                    ProviderFilter::Global => list
                        .into_iter()
                        .filter(|p| p.region == ProviderRegion::Global)
                        .collect(),
                };
                self.providers_json = Self::pretty(&filtered);
            }
            Ok(other) => self.set_error(format!("Unexpected result: {other:?}")),
            Err(e) => self.set_error(format!("load providers failed: {e:?}")),
        }
    }

    pub fn get_provider_config(&mut self) {
        self.clear_error();
        let op = VaultOp::GetProviderConfig {
            provider: self.provider_id.clone(),
        };
        match self.services.api_manager.vault_operation(op) {
            Ok(VaultResult::ProviderConfig(cfg)) => {
                self.provider_config_json = Self::pretty(&cfg);
            }
            Ok(other) => self.set_error(format!("Unexpected result: {other:?}")),
            Err(e) => self.set_error(format!("get provider config failed: {e:?}")),
        }
    }

    pub fn set_default_provider(&mut self) {
        self.clear_error();
        let op = VaultOp::SetDefaultProvider {
            provider: self.provider_id.clone(),
        };
        match self.services.api_manager.vault_operation(op) {
            Ok(VaultResult::DefaultProvider(p)) => {
                self.provider_config_json = format!("Default provider set: {p}\n\n{}", self.provider_config_json);
            }
            Ok(other) => self.set_error(format!("Unexpected result: {other:?}")),
            Err(e) => self.set_error(format!("set default provider failed: {e:?}")),
        }
    }

    pub fn discover_nodes(&mut self) {
        self.clear_error();
        let services = self.services.clone();
        let res = self.runtime.block_on(async move { services.resource_manager.discover_nodes().await });
        match res {
            Ok(nodes) => self.resource_json = Self::pretty(&nodes),
            Err(e) => self.set_error(format!("discover_nodes failed: {e:?}")),
        }
    }

    pub fn list_discovered_nodes(&mut self) {
        self.clear_error();
        let services = self.services.clone();
        let nodes = self.runtime.block_on(async move { services.resource_manager.list_discovered_nodes().await });
        self.resource_json = Self::pretty(&nodes);
    }

    pub fn set_remote_access(&mut self) {
        self.clear_error();
        let allow = self.allow_remote_access;
        let services = self.services.clone();
        self.runtime.block_on(async move {
            services.resource_manager.set_remote_access(allow).await;
        });
        self.resource_json = format!("remote_access={allow}\n\n{}", self.resource_json);
    }

    pub fn get_remote_access_status(&mut self) {
        self.clear_error();
        let services = self.services.clone();
        let allow = self.runtime.block_on(async move { services.resource_manager.get_remote_access_status().await });
        self.allow_remote_access = allow;
        self.resource_json = format!("remote_access={allow}\n\n{}", self.resource_json);
    }

    pub fn create_swarm_group(&mut self) {
        self.clear_error();
        let name = self.group_name.clone();
        let max_members = self.group_max_members;
        let services = self.services.clone();
        let res = self.runtime.block_on(async move {
            services.resource_manager.create_swarm_group(name, max_members).await
        });
        match res {
            Ok(id) => {
                self.group_id = id.clone();
                self.resource_json = format!("created group: {id}\n\n{}", self.resource_json);
            }
            Err(e) => self.set_error(format!("create_swarm_group failed: {e:?}")),
        }
    }

    pub fn join_swarm_group(&mut self) {
        self.clear_error();
        let group_id = self.group_id.clone();
        let services = self.services.clone();
        let res = self.runtime.block_on(async move { services.resource_manager.join_swarm_group(group_id).await });
        if let Err(e) = res {
            self.set_error(format!("join_swarm_group failed: {e:?}"));
        }
    }

    pub fn leave_swarm_group(&mut self) {
        self.clear_error();
        let group_id = self.group_id.clone();
        let services = self.services.clone();
        let res = self.runtime.block_on(async move { services.resource_manager.leave_swarm_group(group_id).await });
        if let Err(e) = res {
            self.set_error(format!("leave_swarm_group failed: {e:?}"));
        }
    }

    pub fn list_swarm_groups(&mut self) {
        self.clear_error();
        let services = self.services.clone();
        let groups = self.runtime.block_on(async move { services.resource_manager.list_swarm_groups().await });
        self.resource_json = Self::pretty(&groups);
    }

    pub fn get_group_members(&mut self) {
        self.clear_error();
        let group_id = self.group_id.clone();
        let services = self.services.clone();
        let res = self.runtime.block_on(async move { services.resource_manager.get_group_members(group_id).await });
        match res {
            Ok(members) => self.resource_json = Self::pretty(&members),
            Err(e) => self.set_error(format!("get_group_members failed: {e:?}")),
        }
    }

    pub fn set_balancing_strategy(&mut self) {
        self.clear_error();
        let strategy = self.balancing_strategy.clone();
        let services = self.services.clone();
        self.runtime.block_on(async move {
            services.resource_manager.set_balancing_strategy(strategy).await;
        });
    }

    pub fn get_balancing_strategy(&mut self) {
        self.clear_error();
        let services = self.services.clone();
        let s = self.runtime.block_on(async move { services.resource_manager.get_balancing_strategy().await });
        self.balancing_strategy = s;
        self.resource_json = format!("balancing={:?}\n\n{}", self.balancing_strategy, self.resource_json);
    }

    pub fn create_resource_pool(&mut self) {
        self.clear_error();
        let name = self.pool_name.clone();
        let node_ids: Vec<String> = self
            .pool_node_ids_csv
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();
        let services = self.services.clone();
        let res = self.runtime.block_on(async move {
            services.resource_manager.create_resource_pool(name, node_ids).await
        });
        match res {
            Ok(id) => self.resource_json = format!("created pool: {id}\n\n{}", self.resource_json),
            Err(e) => self.set_error(format!("create_resource_pool failed: {e:?}")),
        }
    }

    pub fn list_resource_pools(&mut self) {
        self.clear_error();
        let services = self.services.clone();
        let pools = self.runtime.block_on(async move { services.resource_manager.list_resource_pools().await });
        self.resource_json = Self::pretty(&pools);
    }

    pub fn request_resources(&mut self) {
        self.clear_error();

        let cpu_cores = self.req_cpu_cores.trim().parse::<u32>().ok();
        let memory_mb = self.req_memory_mb.trim().parse::<u64>().ok();
        let gpu_memory_mb = self.req_gpu_memory_mb.trim().parse::<u64>().ok();
        let preferred_models: Vec<String> = self
            .req_preferred_models_csv
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();

        let requirements = ResourceRequirements {
            cpu_cores,
            memory_mb,
            gpu_required: self.req_gpu_required,
            gpu_memory_mb,
            preferred_models,
        };

        let task_type = self.req_task_type.clone();
        let priority = self.req_priority.clone();
        let services = self.services.clone();

        let res = self.runtime.block_on(async move {
            services
                .resource_manager
                .request_resources(requirements, task_type, priority)
                .await
        });

        match res {
            Ok(allocation) => {
                self.allocation_id = allocation.allocation_id.clone();
                self.resource_json = Self::pretty(&allocation);
            }
            Err(e) => self.set_error(format!("request_resources failed: {e:?}")),
        }
    }

    pub fn release_allocation(&mut self) {
        self.clear_error();
        let allocation_id = self.allocation_id.clone();
        let services = self.services.clone();
        let res = self.runtime.block_on(async move {
            services.resource_manager.release_allocation(allocation_id).await
        });
        if let Err(e) = res {
            self.set_error(format!("release_allocation failed: {e:?}"));
        }
    }

    pub fn perform_health_check(&mut self) {
        self.clear_error();
        let node_id = self.node_id.clone();
        let services = self.services.clone();
        let res = self.runtime.block_on(async move {
            services.resource_manager.perform_health_check(node_id).await
        });
        match res {
            Ok(hc) => self.resource_json = Self::pretty(&hc),
            Err(e) => self.set_error(format!("health_check failed: {e:?}")),
        }
    }
}
