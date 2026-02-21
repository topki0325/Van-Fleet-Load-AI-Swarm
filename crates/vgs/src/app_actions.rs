use crate::app::VgaGuiApp;
use crate::app_types::{AiEntity, ProviderFilter};
use vangriten_ai_swarm::shared::models::*;

impl VgaGuiApp {
    fn find_entity_by_name(&self, name: &str) -> Option<AiEntity> {
        self.ai_entities.iter().find(|e| e.name == name).cloned()
    }

    fn sanitize_project_name(name: &str) -> String {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return "project".to_string();
        }
        trimmed
            .chars()
            .map(|c| match c {
                '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '-',
                _ => c,
            })
            .collect::<String>()
    }

    /// Create an "Article Quick-Write" project on disk and submit tasks.
    ///
    /// Notes:
    /// - Current scheduler doesn't route to a specific entity; we embed entity names in prompts.
    /// - Files are created as placeholders under `{root}/{project_name}/`.
    pub fn create_article_quick_project(&mut self) {
        self.clear_error();

        let root_dir = self.new_project_root_dir.trim();
        let project_name = Self::sanitize_project_name(&self.new_project_name);
        let topic = self.article_topic.trim();

        if root_dir.is_empty() {
            self.api_quick_status = "❌ 请选择项目目录".to_string();
            return;
        }
        if topic.is_empty() {
            self.api_quick_status = "❌ 请输入文章主题".to_string();
            return;
        }

        if self.article_selected_entities.is_empty() {
            self.api_quick_status = "❌ 至少选择 1 个模型实体".to_string();
            return;
        }
        if self.article_outline_entity.trim().is_empty() {
            self.api_quick_status = "❌ 请选择目录实体".to_string();
            return;
        }
        if self.article_master_entity.trim().is_empty() {
            self.api_quick_status = "❌ 请选择主拼合实体".to_string();
            return;
        }

        let outline_entity = self.article_outline_entity.trim().to_string();
        let master_entity = self.article_master_entity.trim().to_string();

        if !self.article_selected_entities.contains(&outline_entity)
            || !self.article_selected_entities.contains(&master_entity)
        {
            self.api_quick_status = "❌ 目录实体/主实体必须在参与实体列表中".to_string();
            return;
        }

        let groups_count = self.article_groups_count.clamp(1, 10) as usize;

        let root = std::path::PathBuf::from(root_dir);
        let project_dir = root.join(&project_name);
        if let Err(e) = std::fs::create_dir_all(&project_dir) {
            self.api_quick_status = format!("❌ 创建项目目录失败: {e}");
            return;
        }

        // Create placeholder files
        let outline_path = project_dir.join("outline.md");
        let final_path = project_dir.join("final.md");
        let _ = std::fs::write(
            &outline_path,
            format!("# 目录 / Outline\n\n主题: {topic}\n\n(由实体 {outline_entity} 生成)\n"),
        );
        let _ = std::fs::write(
            &final_path,
            format!("# 成稿 / Final\n\n主题: {topic}\n\n(由实体 {master_entity} 拼合)\n"),
        );

        let mut group_files: Vec<std::path::PathBuf> = Vec::new();
        for i in 1..=groups_count {
            let p = project_dir.join(format!("group-{i}.md"));
            let _ = std::fs::write(&p, format!("# 小组 {i}\n\n主题: {topic}\n\n"));
            group_files.push(p);
        }

        // Deploy a lightweight Project record in memory
        let services = self.services.clone();
        let master_provider = self
            .find_entity_by_name(&master_entity)
            .map(|e| e.provider)
            .unwrap_or_else(|| "local".to_string());

        let project_id = self.runtime.block_on(async move {
            let pid = uuid::Uuid::new_v4();
            let project = Project {
                id: pid,
                name: project_name.clone(),
                config: ProjectConfig {
                    tech_stack: vec!["markdown".to_string()],
                    default_provider: master_provider,
                    concurrency_strategy: "article-quick".to_string(),
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
            pid
        });

        // Build group assignments (ensure length)
        let mut assignments = self.article_group_assignments.clone();
        if assignments.len() != groups_count {
            assignments = vec![master_entity.clone(); groups_count];
        }

        let selected_list = self.article_selected_entities.clone();

        // Submit tasks: 1) outline 2) each group section 3) merge
        let services = self.services.clone();
        let topic_owned = topic.to_string();
        let outline_path_str = outline_path.display().to_string();
        let final_path_str = final_path.display().to_string();
        let group_paths_str: Vec<String> = group_files.iter().map(|p| p.display().to_string()).collect();

        let submit_res = self.runtime.block_on(async move {
            let mut submitted: Vec<String> = Vec::new();

            let outline_prompt = format!(
                "任务: 文章快速写(1/3)-写目录\n主题: {topic_owned}\n使用实体: {outline_entity}\n参与实体: {selected_list:?}\n输出要求: 生成详细目录(包含小节标题与要点)。\n请将结果写入文件: {outline_path_str}\n"
            );
            let outline_task = Task::new(
                TaskSpec {
                    language: "markdown".to_string(),
                    target: "article-outline".to_string(),
                    context_range: outline_prompt,
                },
                Priority::High,
                std::path::PathBuf::from("snapshots/gui.json"),
            );
            if let Ok(id) = services.agent_scheduler.submit_task(outline_task).await {
                submitted.push(format!("outline:{id}"));
            }

            for (idx, path) in group_paths_str.iter().enumerate() {
                let group_no = idx + 1;
                let writer = assignments.get(idx).cloned().unwrap_or_else(|| master_entity.clone());
                let prompt = format!(
                    "任务: 文章快速写(2/3)-小组写作\n主题: {topic_owned}\n小组: {group_no}\n使用实体: {writer}\n参考目录文件: {outline_path_str}\n输出要求: 根据目录写出本组负责的小节内容，尽量完整。\n请将结果写入文件: {path}\n"
                );
                let t = Task::new(
                    TaskSpec {
                        language: "markdown".to_string(),
                        target: format!("article-group-{group_no}"),
                        context_range: prompt,
                    },
                    Priority::Medium,
                    std::path::PathBuf::from("snapshots/gui.json"),
                );
                if let Ok(id) = services.agent_scheduler.submit_task(t).await {
                    submitted.push(format!("group-{group_no}:{id}"));
                }
            }

            let merge_prompt = format!(
                "任务: 文章快速写(3/3)-拼合成稿\n主题: {topic_owned}\n主实体: {master_entity}\n目录文件: {outline_path_str}\n小组文件: {group_paths_str:?}\n输出要求: 将目录与各组内容拼合成一篇连贯文章，统一风格与口吻，修正重复/冲突。\n请将结果写入文件: {final_path_str}\n"
            );
            let merge_task = Task::new(
                TaskSpec {
                    language: "markdown".to_string(),
                    target: "article-merge".to_string(),
                    context_range: merge_prompt,
                },
                Priority::High,
                std::path::PathBuf::from("snapshots/gui.json"),
            );
            if let Ok(id) = services.agent_scheduler.submit_task(merge_task).await {
                submitted.push(format!("merge:{id}"));
            }

            submitted
        });

        if submit_res.is_empty() {
            self.api_quick_status = format!("✅ 已创建项目: {project_dir:?} (project {project_id})\n⚠ 未返回任务ID (scheduler可能未启动)");
        } else {
            self.tasks_json = format!(
                "Created project: {} (id: {})\nSubmitted tasks: {:?}\n\n{}",
                project_dir.display(),
                project_id,
                submit_res,
                self.tasks_json
            );
            self.api_quick_status = format!("✅ 已创建项目并提交任务: {}", project_dir.display());
        }

        self.refresh_all();
    }

    /// Create a prototype quick-build project (Website/Software/Game).
    pub fn create_prototype_quick_project(&mut self) {
        self.clear_error();

        let root_dir = self.new_project_root_dir.trim();
        let project_name = Self::sanitize_project_name(&self.new_project_name);
        let topic = self.article_topic.trim(); // Reusing the same topic field
        let kind = self.new_project_kind.clone();

        if root_dir.is_empty() {
            self.api_quick_status = "❌ 请选择项目目录".to_string();
            return;
        }
        if topic.is_empty() {
            self.api_quick_status = "❌ 请输入项目目标/概念".to_string();
            return;
        }
        if self.article_selected_entities.is_empty() {
            self.api_quick_status = "❌ 至少选择 1 个模型实体".to_string();
            return;
        }

        let arch_entity = self.article_outline_entity.trim().to_string(); // Reused label
        let master_entity = self.article_master_entity.trim().to_string();
        let groups_count = self.article_groups_count.clamp(1, 10) as usize;

        // Build assignments
        let mut assignments = self.article_group_assignments.clone();
        if assignments.len() != groups_count {
            assignments = vec![master_entity.clone(); groups_count];
        }

        let root = std::path::PathBuf::from(root_dir);
        let project_dir = root.join(&project_name);
        if let Err(e) = std::fs::create_dir_all(&project_dir) {
            self.api_quick_status = format!("❌ 创建项目目录失败: {e}");
            return;
        }

        // Contextual file structure and specialized configurations
        let (spec_file, core_dir, final_file, lang, prompt_extra) = match kind.as_str() {
            "网站原型快速搭建" => (
                "design_spec.md",
                "components",
                "index.html",
                "HTML5/Tailwind/JS",
                "输出要求: 包含响应式布局、交互原型、色彩方案和组件列表。所有静态资源引用 CDN。"
            ),
            "软件原型快速搭建" => (
                "architecture.md",
                "src",
                "main.rs",
                "Rust",
                "输出要求: 定义核心 Data Types, Traits/Interfaces, 和主功能逻辑流程。遵循简洁高效的原则。"
            ),
            "游戏原型快速搭建" => (
                "game_design.md",
                "scripts",
                "index.html",
                "Phaser.js",
                "输出要求: 包含游戏循环逻辑、物理系统参数、输入响应映射和核心关卡/玩法设计草案。"
            ),
            _ => ("spec.md", "logic", "output.txt", "Text", "默认模式")
        };

        let spec_path = project_dir.join(spec_file);
        let core_path = project_dir.join(core_dir);
        let final_path = project_dir.join(final_file);
        let _ = std::fs::create_dir_all(&core_path);

        let intro = format!("# {kind} - {project_name}\n\n## 愿景/目标\n{topic}\n\n## 参与实体\n- 架构师/设计师: {arch_entity}\n- 核心开发者: {assignments:?}\n- 整合专家: {master_entity}\n\n");
        let _ = std::fs::write(&spec_path, format!("{intro}## 规格说明\n(待生成...)\n"));
        let _ = std::fs::write(&final_path, format!("<!-- PROTOTYPE: {kind} -->\n<!-- 这是一个自动生成的新加坡项目原型占位符 -->\n"));

        // Deploy a Project record
        let services = self.services.clone();
        let master_provider = self.find_entity_by_name(&master_entity).map(|e| e.provider).unwrap_or_else(|| "local".to_string());

        let project_id = self.runtime.block_on(async move {
            let pid = uuid::Uuid::new_v4();
            let project = Project {
                id: pid,
                name: project_name.clone(),
                config: ProjectConfig {
                    tech_stack: vec![lang.to_string()],
                    default_provider: master_provider,
                    concurrency_strategy: "prototype-quick".to_string(),
                },
                agents: Vec::new(),
                workflow: WorkflowGraph::default(),
                state: ProjectStatus::Initialized,
                stats: ExecutionStats::default(),
                last_updated: chrono::Utc::now(),
            };
            services.projects.write().await.push(project);
            pid
        });

        // Submit tasks (Architecture -> Components -> Integration)
        let services_clone = self.services.clone();
        let topic_owned = topic.to_string();
        let spec_path_str = spec_path.display().to_string();
        let core_path_str = core_path.display().to_string();
        let final_path_str = final_path.display().to_string();

        let submit_res = self.runtime.block_on(async move {
            let mut submitted = Vec::new();

            // Task 1: Architecture/Design
            let prompt_arch = format!(
                "模式: {kind} (1/3)-架构设计\n目标: {topic_owned}\n建议技术栈: {lang}\n使用实体: {arch_entity}\n{prompt_extra}\n直接修改并保存至: {spec_path_str}\n"
            );
            let t1 = Task::new(TaskSpec { language: lang.to_string(), target: "design".to_string(), context_range: prompt_arch }, Priority::High, std::path::PathBuf::from("snapshots/gui.json"));
            if let Ok(id) = services_clone.agent_scheduler.submit_task(t1).await { submitted.push(format!("design:{id}")); }

            // Task 2: Component implementation (parallel)
            for i in 1..=groups_count {
                let dev_entity = assignments.get(i-1).cloned().unwrap_or(master_entity.clone());
                let prompt_comp = format!(
                    "模式: {kind} (2/3)-模块实现 (任务单元 {i})\n目标: {topic_owned}\n参考架构文件: {spec_path_str}\n当前执行单元: {dev_entity}\n输出要求: 编写核心逻辑代码或 UI 模块，并保存在目录 {core_path_str} 下对应的文件名中。\n"
                );
                let ti = Task::new(TaskSpec { language: lang.to_string(), target: format!("module-{i}"), context_range: prompt_comp }, Priority::Medium, std::path::PathBuf::from("snapshots/gui.json"));
                if let Ok(id) = services_clone.agent_scheduler.submit_task(ti).await { submitted.push(format!("module-{i}:{id}")); }
            }

            // Task 3: Final Integration
            let prompt_merge = format!(
                "模式: {kind} (3/3)-原型整合\n目标: {topic_owned}\n整合实体: {master_entity}\n输入: {spec_path_str} 与 {core_path_str} 中的所有内容\n输出要求: 将所有零散模块和设计整合进一个单一的、可运行预览的文件 {final_file} 中。如果是网站/游戏，请提供完整的静态 HTML/JS 代码；如果是软件，请提供主入口 main 逻辑。\n保存至: {final_path_str}\n"
            );
            let t3 = Task::new(TaskSpec { language: lang.to_string(), target: "integration".to_string(), context_range: prompt_merge }, Priority::High, std::path::PathBuf::from("snapshots/gui.json"));
            if let Ok(id) = services_clone.agent_scheduler.submit_task(t3).await { submitted.push(format!("integration:{id}")); }

            submitted
        });

        self.tasks_json = format!(
            "Created Prototype Project: {} (id: {})\nSubmitted tasks: {:?}\n\n{}",
            project_dir.display(),
            project_id,
            submit_res,
            self.tasks_json
        );
        self.api_quick_status = format!("✅ 已创建原型项目并提交任务: {}", project_dir.display());

        self.refresh_all();
    }

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

    /// Refresh the cached list of stored provider names (requires vault unlocked).
    pub fn load_stored_keys(&mut self) {
        match self.services.api_manager.vault_operation(VaultOp::List) {
            Ok(VaultResult::Providers(list)) => {
                // Filter out internal vault metadata entries
                self.api_stored_providers = list
                    .into_iter()
                    .filter(|k| k != "salt" && k != "vault_check")
                    .collect();
                self.api_list_json = Self::pretty(&self.api_stored_providers);
            }
            Ok(other) => self.api_quick_status = format!("Unexpected: {other:?}"),
            Err(e) => self.api_quick_status = format!("list failed: {e:?}"),
        }
    }

    /// Load async usage statistics into `api_usage_json`.
    pub fn load_usage_stats(&mut self) {
        let services = self.services.clone();
        let entries = self
            .runtime
            .block_on(async move { services.api_manager.get_usage_entries().await });
        self.api_usage_json = Self::pretty(&entries);
    }

    // ── AI Entity management ──────────────────────────────────────────────────

    fn entities_path() -> std::path::PathBuf {
        std::path::PathBuf::from("vault/entities.json")
    }

    /// Load named AI entities from disk.
    pub fn load_entities(&mut self) {
        let path = Self::entities_path();
        if !path.exists() {
            self.ai_entities = Vec::new();
            return;
        }
        match std::fs::read_to_string(&path) {
            Ok(s) => {
                self.ai_entities = serde_json::from_str(&s).unwrap_or_default();
            }
            Err(e) => self.api_quick_status = format!("load entities failed: {e}"),
        }
    }

    /// Persist named AI entities to disk.
    pub fn save_entities(&mut self) {
        let path = Self::entities_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        match serde_json::to_string_pretty(&self.ai_entities) {
            Ok(s) => {
                if let Err(e) = std::fs::write(&path, s) {
                    self.api_quick_status = format!("save entities failed: {e}");
                }
            }
            Err(e) => self.api_quick_status = format!("serialize entities failed: {e}"),
        }
    }

    // ── Custom provider management ────────────────────────────────────────

    fn custom_providers_path() -> std::path::PathBuf {
        std::path::PathBuf::from("vault/custom_providers.json")
    }

    /// Load user-defined custom/relay providers from disk.
    pub fn load_custom_providers(&mut self) {
        let path = Self::custom_providers_path();
        if !path.exists() {
            self.custom_providers = Vec::new();
            return;
        }
        match std::fs::read_to_string(&path) {
            Ok(s) => self.custom_providers = serde_json::from_str(&s).unwrap_or_default(),
            Err(e) => self.api_quick_status = format!("load custom providers failed: {e}"),
        }
    }

    /// Persist custom/relay providers to disk.
    pub fn save_custom_providers(&mut self) {
        let path = Self::custom_providers_path();
        if let Some(p) = path.parent() { let _ = std::fs::create_dir_all(p); }
        match serde_json::to_string_pretty(&self.custom_providers) {
            Ok(s) => { if let Err(e) = std::fs::write(&path, s) {
                self.api_quick_status = format!("save custom providers failed: {e}");
            }}
            Err(e) => self.api_quick_status = format!("serialize custom providers failed: {e}"),
        }
    }

    /// Add or update a custom/relay provider.
    pub fn add_custom_provider(&mut self) {
        let id = self.cp_id_input.trim().to_string();
        let name = self.cp_name_input.trim().to_string();
        let base_url = self.cp_url_input.trim().to_string();
        if id.is_empty() || base_url.is_empty() {
            self.api_quick_status = "❌ 供应商 ID 和地址不能为空".to_string();
            return;
        }
        let cp = crate::app_types::CustomProvider {
            id: id.clone(),
            name: if name.is_empty() { id.clone() } else { name },
            base_url,
            key_header: self.cp_key_header_input.trim().to_string(),
            key_prefix: self.cp_key_prefix_input.trim().to_string(),
            models_hint: self.cp_models_input.trim().to_string(),
            description: self.cp_description_input.trim().to_string(),
        };
        if let Some(pos) = self.custom_providers.iter().position(|p| p.id == id) {
            self.custom_providers[pos] = cp;
        } else {
            self.custom_providers.push(cp);
        }
        self.save_custom_providers();
        self.cp_id_input.clear();
        self.cp_name_input.clear();
        self.cp_url_input.clear();
        self.cp_key_header_input.clear();
        self.cp_key_prefix_input.clear();
        self.cp_models_input.clear();
        self.cp_description_input.clear();
        self.api_quick_status = format!("✅ 供应商已保存: {id}");
    }

    /// Delete a custom provider by id.
    pub fn delete_custom_provider(&mut self, id: &str) {
        self.custom_providers.retain(|p| p.id != id);
        self.save_custom_providers();
        self.api_quick_status = format!("✅ 已删除供应商: {id}");
    }

    /// Add or update a named AI entity and store its API key in the vault.
    /// The vault key is stored under `entity.name`.
    pub fn add_entity(&mut self) {
        let name = self.entity_name_input.trim().to_string();
        let provider = self.api_provider.trim().to_string();
        let model = self.entity_model_input.trim().to_string();
        let note = self.entity_note_input.trim().to_string();
        let key = self.api_key_input.trim().to_string();
        let custom_url = self.entity_custom_url_input.trim();
        let key_header = self.entity_key_header_input.trim();
        let key_prefix = self.entity_key_prefix_input.trim();

        if name.is_empty() {
            self.api_quick_status = "❌ 名称不能为空".to_string();
            return;
        }
        if provider.is_empty() {
            self.api_quick_status = "❌ Provider 不能为空".to_string();
            return;
        }
        if key.is_empty() {
            self.api_quick_status = "❌ API Key 不能为空".to_string();
            return;
        }

        // Store key in vault using entity name as the vault key
        let am = self.services.api_manager.clone();
        match am.vault_operation(VaultOp::Store {
            provider: name.clone(),
            key,
        }) {
            Ok(_) => {}
            Err(e) => {
                self.api_quick_status = format!("❌ vault store failed: {e:?}");
                return;
            }
        }

        // Upsert entity metadata
        let entity = AiEntity {
            name: name.clone(),
            provider,
            model,
            note,
            custom_base_url: if custom_url.is_empty() { None } else { Some(custom_url.to_string()) },
            key_header: if key_header.is_empty() { None } else { Some(key_header.to_string()) },
            key_prefix: if key_prefix.is_empty() { None } else { Some(key_prefix.to_string()) },
        };
        if let Some(pos) = self.ai_entities.iter().position(|e| e.name == name) {
            self.ai_entities[pos] = entity;
        } else {
            self.ai_entities.push(entity);
        }
        self.save_entities();

        self.api_key_input.clear();
        self.entity_name_input.clear();
        self.entity_model_input.clear();
        self.entity_note_input.clear();
        self.entity_custom_url_input.clear();
        self.entity_key_header_input.clear();
        self.entity_key_prefix_input.clear();
        self.load_stored_keys();
        self.api_quick_status = format!("✅ 已保存: {name}");
    }

    /// Create `entity_burst_count` copies of the current form entity, named
    /// `{base}-1` … `{base}-N`, all sharing the same API key.
    pub fn burst_add_entities(&mut self) {
        let base = self.entity_name_input.trim().to_string();
        let provider = self.api_provider.trim().to_string();
        let model = self.entity_model_input.trim().to_string();
        let note = self.entity_note_input.trim().to_string();
        let key = self.api_key_input.trim().to_string();
        let custom_url = self.entity_custom_url_input.trim().to_string();
        let key_header = self.entity_key_header_input.trim().to_string();
        let key_prefix = self.entity_key_prefix_input.trim().to_string();
        let count = self.entity_burst_count.clamp(1, 10) as usize;

        if base.is_empty() { self.api_quick_status = "❌ 名称不能为空".to_string(); return; }
        if provider.is_empty() { self.api_quick_status = "❌ Provider 不能为空".to_string(); return; }
        if key.is_empty() { self.api_quick_status = "❌ API Key 不能为空".to_string(); return; }

        let am = self.services.api_manager.clone();
        let mut created = 0usize;
        for i in 1..=count {
            let name = format!("{base}-{i}");
            // Store the same key under each entity name
            match am.vault_operation(VaultOp::Store { provider: name.clone(), key: key.clone() }) {
                Ok(_) => {}
                Err(e) => { self.api_quick_status = format!("❌ vault store {name}: {e:?}"); return; }
            }
            let entity = AiEntity {
                name: name.clone(), provider: provider.clone(), model: model.clone(), note: note.clone(),
                custom_base_url: if custom_url.is_empty() { None } else { Some(custom_url.clone()) },
                key_header: if key_header.is_empty() { None } else { Some(key_header.clone()) },
                key_prefix: if key_prefix.is_empty() { None } else { Some(key_prefix.clone()) },
            };
            if let Some(pos) = self.ai_entities.iter().position(|e| e.name == name) {
                self.ai_entities[pos] = entity;
            } else {
                self.ai_entities.push(entity);
            }
            created += 1;
        }
        self.save_entities();
        self.api_key_input.clear();
        self.entity_name_input.clear();
        self.entity_model_input.clear();
        self.entity_note_input.clear();
        self.entity_custom_url_input.clear();
        self.entity_key_header_input.clear();
        self.entity_key_prefix_input.clear();
        self.load_stored_keys();
        self.api_quick_status = format!("✅ 裂变创建 {created} 个: {base}-1 … {base}-{count}");
    }

    /// Delete a named AI entity and remove its key from the vault.
    pub fn delete_entity(&mut self, name: &str) {
        let am = self.services.api_manager.clone();
        let _ = am.vault_operation(VaultOp::Delete { provider: name.to_string() });
        self.ai_entities.retain(|e| e.name != name);
        self.save_entities();
        self.load_stored_keys();
        self.api_quick_status = format!("🗑 已删除: {name}");
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
