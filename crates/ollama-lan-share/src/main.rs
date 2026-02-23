use std::time::Duration;

mod shared;
mod backend;

use crate::backend::{NetworkDiscovery, OllamaManager};

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args = Args::from_env();

    if args.help {
        print_usage_and_exit();
    }

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("create tokio runtime");

    rt.block_on(async move {
        let discovery = NetworkDiscovery::new().await;


        if let Some(name) = args.name.clone() {
            discovery.set_local_node_name(name).await;
        }

        if !args.groups.is_empty() {
            discovery.set_local_groups(args.groups.clone()).await;
        }

        let local_name = discovery.local_node_name().await;

        // Only touch Ollama if we actually plan to share something.
        let mut share_enabled = false;
        let mut models = Vec::new();
        if args.share_all || args.models_csv.is_some() || args.models_file.is_some() {
            let ollama = OllamaManager::new(args.ollama_base_url.clone()).await;
            (share_enabled, models) = resolve_models(&ollama, &args).await;
        }

        discovery
            .set_ollama_offer(share_enabled, models.clone(), args.ollama_base_url.clone())
            .await;

        tracing::info!("local_name={}", local_name);
        tracing::info!("groups={:?}", args.groups);
        if share_enabled {
            tracing::info!("ollama_share=enabled models={:?}", models);
        } else {
            tracing::info!("ollama_share=disabled");
        }

        discovery.broadcast_presence();

        if args.once {
            // Give the background send task a brief moment to run before we exit.
            tokio::time::sleep(Duration::from_millis(200)).await;
            return;
        }

        let mut interval = tokio::time::interval(Duration::from_secs(5));
        loop {
            interval.tick().await;
            let d = discovery.debug_stats().await;
            tracing::info!(
                "discovery: bound={} tx(a/q)={}/{} rx(a/q)={}/{} last_rx={:?} {:?}",
                d.socket_bound,
                d.sent_announces,
                d.sent_queries,
                d.received_announces,
                d.received_queries,
                d.last_received_kind,
                d.last_received_from
            );
        }
    });
}

fn print_usage_and_exit() -> ! {
    eprintln!(
        "ollama-lan-share\n\nUSAGE:\n  ollama-lan-share [options]\n\nOPTIONS:\n  --help                 Show this help\n  --once                 Broadcast once and exit\n  --name <name>           Override machine name\n  --group <groupId>       Join/advertise a group (repeatable)\n  --groups <csv>          Comma-separated groups\n  --models <csv>          Share subset of local Ollama models\n  --models-file <path>    Share subset listed in a file\n  --share-all             Share all local Ollama models\n  --ollama <base_url>     Ollama base URL (default: http://localhost:11434)\n"
    );
    std::process::exit(0)
}

async fn resolve_models(ollama: &OllamaManager, args: &Args) -> (bool, Vec<String>) {
    let mut requested = Vec::new();
    if let Some(csv) = &args.models_csv {
        requested.extend(split_list(csv));
    }

    if let Some(path) = &args.models_file {
        match std::fs::read_to_string(path) {
            Ok(content) => {
                for line in content.lines() {
                    requested.extend(split_list(line));
                }
            }
            Err(err) => {
                tracing::warn!("Failed to read models file {}: {}", path, err);
            }
        }
    }

    let requested: Vec<String> = requested
        .into_iter()
        .map(|m| m.trim().to_string())
        .filter(|m| !m.is_empty())
        .collect();

    if !requested.is_empty() {
        let local = match ollama.list_models().await {
            Ok(list) => list.into_iter().map(|m| m.name).collect::<Vec<_>>(),
            Err(err) => {
                tracing::warn!("Failed to list local Ollama models: {}", err);
                Vec::new()
            }
        };

        let mut selected: Vec<String> = requested
            .iter()
            .filter(|m| local.contains(m))
            .cloned()
            .collect();
        selected.sort();
        selected.dedup();

        let missing: Vec<String> = requested
            .into_iter()
            .filter(|m| !local.contains(m))
            .collect();
        if !missing.is_empty() {
            tracing::warn!("Requested models not found locally: {:?}", missing);
        }

        return (!selected.is_empty(), selected);
    }

    if !args.share_all {
        return (false, Vec::new());
    }

    match ollama.list_models().await {
        Ok(list) => {
            let mut out: Vec<String> = list.into_iter().map(|m| m.name).collect();
            out.sort();
            out.dedup();
            (!out.is_empty(), out)
        }
        Err(err) => {
            tracing::warn!("Failed to list local Ollama models: {}", err);
            (false, Vec::new())
        }
    }
}

#[derive(Debug, Default, Clone)]
struct Args {
    name: Option<String>,
    groups: Vec<String>,
    models_csv: Option<String>,
    models_file: Option<String>,
    share_all: bool,
    ollama_base_url: Option<String>,
    once: bool,
    help: bool,
}

impl Args {
    fn from_env() -> Self {
        // Minimal parsing:
        //   --name <name>
        //   --group <groupId>   (repeatable)
        //   --groups <csv>
        //   --models <csv>
        //   --models-file <path>
        //   --share-all
        //   --ollama <base_url>
        //   --once
        //   --help
        let mut out = Args::default();
        let mut it = std::env::args().skip(1);
        while let Some(a) = it.next() {
            match a.as_str() {
                "--help" | "-h" => out.help = true,
                "--name" => out.name = it.next(),
                "--group" => {
                    if let Some(g) = it.next() {
                        if !g.trim().is_empty() {
                            out.groups.push(g);
                        }
                    }
                }
                "--groups" => {
                    if let Some(csv) = it.next() {
                        out.groups.extend(split_list(&csv));
                    }
                }
                "--models" => out.models_csv = it.next(),
                "--models-file" => out.models_file = it.next(),
                "--share-all" => out.share_all = true,
                "--ollama" => out.ollama_base_url = it.next(),
                "--once" => out.once = true,
                _ => {}
            }
        }

        out
    }
}

fn split_list(s: &str) -> Vec<String> {
    s.split(',')
        .map(|x| x.trim())
        .filter(|x| !x.is_empty())
        .map(|x| x.to_string())
        .collect()
}
