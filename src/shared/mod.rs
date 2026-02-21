pub mod models;
pub mod utils;

pub async fn prime_shared_usage() {
	use crate::shared::models::*;

	let mut project = Project {
		id: ProjectId::new_v4(),
		name: "demo".to_string(),
		config: ProjectConfig {
			tech_stack: vec!["rust".to_string()],
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

	project.initialize_workflow();
	let _ = project.validate_and_snapshot();

	let agent = Agent {
		id: AgentId::new_v4(),
		role: AgentType::ProgrammerNode,
		status: AgentStatus::Idle,
		skills: SkillVector { skills: vec!["rust".to_string()] },
		current_task: None,
		performance: PerfMetrics {
			cpu_usage: 0.0,
			memory_usage: 0.0,
			avg_response_time: std::time::Duration::from_millis(1),
		},
		heartbeat: chrono::Utc::now(),
	};
	let _ = agent.is_overloaded();

	let mut task = Task::new(
		TaskSpec {
			language: "rust".to_string(),
			target: "code".to_string(),
			context_range: "demo".to_string(),
		},
		Priority::Low,
		std::path::PathBuf::from("snapshots/demo.json"),
	);
	task.finalize_with_result(TaskResult::Success(TaskOutput::default()));
	let _ = task.check_dependencies(&WorkflowGraph::default());

	let gatling_state = GatlingState {
		available_pool: Vec::new(),
		rotation_index: std::sync::atomic::AtomicUsize::new(0),
		max_concurrency: 1,
		waiting_queue: Vec::new(),
	};
	let _ = gatling_state.available_pool.len();
	let _ = gatling_state.rotation_index.load(std::sync::atomic::Ordering::SeqCst);
	let _ = gatling_state.max_concurrency;
	let _ = gatling_state.waiting_queue.len();

	let _ = Snapshot {
		project_id: project.id,
		timestamp: chrono::Utc::now(),
		data: Vec::new(),
	};

	let _ = CodeSpec {
		language: "rust".to_string(),
		requirements: vec!["demo".to_string()],
		context: "demo".to_string(),
	};

	let handle = tokio::spawn(async { Ok(TaskOutput::default()) });
	let task_handle = TaskHandle::new(TaskId::new_v4(), handle);
	let _ = task_handle.task_id;
	let _ = &task_handle.handle;

	let build_plan = BuildPlan {
		project: project.clone(),
		segments: vec![BuildSegment {
			language: "rust".to_string(),
			files: vec![std::path::PathBuf::from("src/main.rs")],
		}],
	};
	let _ = build_plan.project.name;
	let _ = build_plan.segments.len();
	if let Some(segment) = build_plan.segments.first() {
		let _ = &segment.language;
		let _ = segment.files.len();
	}

	let _ = utils::compute_hash(b"demo");
	let _ = utils::generate_id();
	let _ = utils::format_duration(std::time::Duration::from_secs(1));
	let _ = utils::parse_timestamp("2026-02-21T00:00:00Z");
}