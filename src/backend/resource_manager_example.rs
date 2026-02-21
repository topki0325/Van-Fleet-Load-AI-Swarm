use crate::backend::resource_manager::ResourceManager;
use crate::shared::models::{ResourceRequirements, Priority, BalancingStrategy};

pub async fn resource_manager_example() {
    let resource_manager = ResourceManager::new(true).await.unwrap();

    resource_manager.start_discovery().await.unwrap();

    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    let discovered = resource_manager.discover_nodes().await.unwrap();
    println!("Discovered {} nodes", discovered.len());

    let group_id = resource_manager.create_swarm_group("AI Swarm Group".to_string(), 10).await.unwrap();
    println!("Created swarm group: {}", group_id);

    let requirements = ResourceRequirements {
        cpu_cores: Some(2),
        memory_mb: Some(4096),
        gpu_required: false,
        gpu_memory_mb: None,
        preferred_models: vec!["gpt-4".to_string()],
    };

    let allocation = resource_manager.request_resources(
        requirements,
        "inference".to_string(),
        Priority::High,
    ).await.unwrap();

    println!("Allocated resources on node: {}", allocation.node_id);

    resource_manager.set_balancing_strategy(BalancingStrategy::LeastLoaded).await;

    let health = resource_manager.perform_health_check(allocation.node_id).await.unwrap();
    println!("Health check - Status: {:?}, Response time: {}ms", health.status, health.response_time_ms);

    resource_manager.release_allocation(allocation.allocation_id).await.unwrap();
}