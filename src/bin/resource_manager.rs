use std::env;
use std::io::{self, Write};
use tokio::time::{sleep, Duration};
use vangriten_ai_swarm::backend::ResourceManager;
use vangriten_ai_swarm::shared::models::{ResourceRequirements, Priority, BalancingStrategy};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║     Vangriten AI Swarm - Resource Manager CLI              ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();

    let args: Vec<String> = env::args().collect();
    
    if args.len() > 1 {
        handle_command(&args).await;
    } else {
        run_interactive().await;
    }
}

async fn handle_command(args: &[String]) {
    let command = &args[1];
    
    match command.as_str() {
        "discover" => {
            println!("Discovering nodes in the network...");
            run_discovery().await;
        }
        "status" => {
            println!("Checking resource manager status...");
            run_status().await;
        }
        "help" => {
            print_help();
        }
        _ => {
            println!("Unknown command: {}", command);
            print_help();
        }
    }
}

async fn run_interactive() {
    let resource_manager = ResourceManager::new(true).await
        .expect("Failed to initialize resource manager");

    println!("Starting resource manager...");
    resource_manager.start_discovery().await
        .expect("Failed to start discovery");

    println!("Resource manager started successfully!");
    println!("Type 'help' for available commands, 'quit' to exit.");
    println!();

    loop {
        print!("resource-manager> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read input");
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        if input == "quit" || input == "exit" {
            println!("Shutting down resource manager...");
            break;
        }

        match input {
            "help" => print_help(),
            "discover" => {
                println!("\nDiscovering nodes...");
                match resource_manager.discover_nodes().await {
                    Ok(nodes) => {
                        println!("Found {} node(s):", nodes.len());
                        for node in nodes {
                            println!("  - {} @ {}:{} (Status: {:?})",
                                node.id, node.address, node.port, node.status);
                            println!("    CPU: {} cores, Memory: {} MB, GPUs: {}",
                                node.resources.cpu_cores,
                                node.resources.available_memory_mb,
                                node.resources.gpus.len());
                        }
                    }
                    Err(e) => println!("Error: {:?}", e),
                }
            }
            "list" => {
                println!("\nDiscovered nodes:");
                let nodes = resource_manager.list_discovered_nodes().await;
                for node in nodes {
                    println!("  - {} @ {}:{} (Status: {:?})",
                        node.id, node.address, node.port, node.status);
                }
            }
            "groups" => {
                println!("\nSwarm groups:");
                let groups = resource_manager.list_swarm_groups().await;
                for group in groups {
                    println!("  - {} ({}) - {} members",
                        group.name, group.group_id, group.members.len());
                }
            }
            "pools" => {
                println!("\nResource pools:");
                let pools = resource_manager.list_resource_pools().await;
                for pool in pools {
                    println!("  - {} ({}) - {} nodes",
                        pool.name, pool.pool_id, pool.nodes.len());
                }
            }
            "create-group" => {
                print!("Enter group name: ");
                io::stdout().flush().unwrap();
                let mut name = String::new();
                io::stdin().read_line(&mut name).expect("Failed to read input");
                let name = name.trim();

                print!("Enter max members: ");
                io::stdout().flush().expect("Failed to flush stdout");
                let mut max = String::new();
                io::stdin().read_line(&mut max).expect("Failed to read input");
                let max_members: usize = max.trim().parse().unwrap_or(10).min(1000);

                if max_members == 0 {
                    println!("Error: max members must be at least 1");
                    continue;
                }

                match resource_manager.create_swarm_group(name.to_string(), max_members).await {
                    Ok(group_id) => println!("Created group: {}", group_id),
                    Err(e) => println!("Error: {:?}", e),
                }
            }
            "request" => {
                print!("Enter CPU cores (default: 2): ");
                io::stdout().flush().expect("Failed to flush stdout");
                let mut cpu = String::new();
                io::stdin().read_line(&mut cpu).expect("Failed to read input");
                let cpu_cores: Option<u32> = cpu.trim().parse().ok().filter(|&x| x > 0 && x <= 128);

                print!("Enter memory in MB (default: 4096): ");
                io::stdout().flush().expect("Failed to flush stdout");
                let mut mem = String::new();
                io::stdin().read_line(&mut mem).expect("Failed to read input");
                let memory_mb: Option<u64> = mem.trim().parse().ok().filter(|&x| x >= 512 && x <= 1024 * 1024);

                if cpu_cores.is_none() || memory_mb.is_none() {
                    println!("Error: Invalid input values");
                    continue;
                }

                print!("GPU required? (y/n, default: n): ");
                io::stdout().flush().expect("Failed to flush stdout");
                let mut gpu = String::new();
                io::stdin().read_line(&mut gpu).expect("Failed to read input");
                let gpu_required = gpu.trim().to_lowercase() == "y";

                let requirements = ResourceRequirements {
                    cpu_cores,
                    memory_mb,
                    gpu_required,
                    gpu_memory_mb: if gpu_required { Some(8192) } else { None },
                    preferred_models: vec![],
                };

                match resource_manager.request_resources(
                    requirements,
                    "cli-request".to_string(),
                    Priority::Medium,
                ).await {
                    Ok(allocation) => {
                        println!("Resources allocated!");
                        println!("  Allocation ID: {}", allocation.allocation_id);
                        println!("  Node: {}", allocation.node_id);
                        println!("  CPU: {} cores", allocation.allocated_resources.cpu_cores);
                        println!("  Memory: {} MB", allocation.allocated_resources.memory_mb);
                        if let Some(gpu) = allocation.allocated_resources.gpu {
                            println!("  GPU: {} ({} MB)", gpu.gpu_id, gpu.memory_mb);
                        }
                    }
                    Err(e) => println!("Error: {:?}", e),
                }
            }
            "strategy" => {
                println!("\nCurrent balancing strategy: {:?}", 
                    resource_manager.get_balancing_strategy().await);
                println!("Available strategies:");
                println!("  1. LeastLoaded");
                println!("  2. RoundRobin");
                println!("  3. Weighted");
                println!("  4. Geographic");
                println!("  5. Random");
            }
            "set-strategy" => {
                print!("Enter strategy number (1-5): ");
                io::stdout().flush().expect("Failed to flush stdout");
                let mut strat = String::new();
                io::stdin().read_line(&mut strat).expect("Failed to read input");
                
                let strategy = match strat.trim() {
                    "1" => BalancingStrategy::LeastLoaded,
                    "2" => BalancingStrategy::RoundRobin,
                    "3" => BalancingStrategy::Weighted,
                    "4" => BalancingStrategy::Geographic,
                    "5" => BalancingStrategy::Random,
                    _ => {
                        println!("Invalid strategy");
                        continue;
                    }
                };
                
                resource_manager.set_balancing_strategy(strategy).await;
                println!("Strategy updated!");
            }
            "remote" => {
                let status = resource_manager.get_remote_access_status().await;
                println!("Remote access: {}", status);
                print!("Toggle remote access? (y/n): ");
                io::stdout().flush().expect("Failed to flush stdout");
                let mut ans = String::new();
                io::stdin().read_line(&mut ans).expect("Failed to read input");
                
                if ans.trim().to_lowercase() == "y" {
                    resource_manager.set_remote_access(!status).await;
                    println!("Remote access: {}", !status);
                }
            }
            "health" => {
                print!("Enter node ID: ");
                io::stdout().flush().expect("Failed to flush stdout");
                let mut node_id = String::new();
                io::stdin().read_line(&mut node_id).expect("Failed to read input");
                
                match resource_manager.perform_health_check(node_id.trim().to_string()).await {
                    Ok(health) => {
                        println!("Health check result:");
                        println!("  Node: {}", health.node_id);
                        println!("  Status: {:?}", health.status);
                        println!("  Response time: {} ms", health.response_time_ms);
                        println!("  Active allocations: {}", health.active_allocations);
                        println!("  CPU load: {:.1}%", health.resources.current_load * 100.0);
                    }
                    Err(e) => println!("Error: {:?}", e),
                }
            }
            "clear" => {
                print!("\x1B[2J\x1B[1;1H");
                io::stdout().flush().expect("Failed to flush stdout");
                println!("╔════════════════════════════════════════════════════════════╗");
                println!("║     Vangriten AI Swarm - Resource Manager CLI              ║");
                println!("╚════════════════════════════════════════════════════════════╝");
                println!();
            }
            _ => println!("Unknown command. Type 'help' for available commands."),
        }
        
        println!();
    }
}

async fn run_discovery() {
    let resource_manager = ResourceManager::new(true).await
        .expect("Failed to initialize resource manager");

    resource_manager.start_discovery().await
        .expect("Failed to start discovery");

    sleep(Duration::from_secs(3)).await;

    match resource_manager.discover_nodes().await {
        Ok(nodes) => {
            println!("Found {} node(s):", nodes.len());
            for node in nodes {
                println!("  - {} @ {}:{} (Status: {:?})",
                    node.id, node.address, node.port, node.status);
                println!("    CPU: {} cores, Memory: {} MB, GPUs: {}",
                    node.resources.cpu_cores,
                    node.resources.available_memory_mb,
                    node.resources.gpus.len());
            }
        }
        Err(e) => println!("Error: {:?}", e),
    }
}

async fn run_status() {
    let resource_manager = ResourceManager::new(true).await
        .expect("Failed to initialize resource manager");

    println!("Resource Manager Status:");
    println!("  Remote access: {}", resource_manager.get_remote_access_status().await);
    println!("  Balancing strategy: {:?}", resource_manager.get_balancing_strategy().await);
    println!("  Swarm groups: {}", resource_manager.list_swarm_groups().await.len());
    println!("  Resource pools: {}", resource_manager.list_resource_pools().await.len());
}

fn print_help() {
    println!("\nAvailable commands:");
    println!("  help          - Show this help message");
    println!("  discover      - Discover nodes in the network");
    println!("  list          - List all discovered nodes");
    println!("  groups        - List all swarm groups");
    println!("  pools         - List all resource pools");
    println!("  create-group  - Create a new swarm group");
    println!("  request       - Request resource allocation");
    println!("  strategy      - Show current balancing strategy");
    println!("  set-strategy - Set balancing strategy");
    println!("  remote        - Toggle remote access");
    println!("  health        - Perform health check on a node");
    println!("  clear         - Clear the screen");
    println!("  quit/exit     - Exit the program");
    println!();
}