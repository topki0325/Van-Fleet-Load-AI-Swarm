use std::time::Duration;

use vas_core::backend::{NetworkDiscovery, OllamaManager};

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args = Args::from_env();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("create tokio runtime");

    rt.block_on(async move {
        let discovery = NetworkDiscovery::new().await;

        if let Some(name) = args.name {
            discovery.set_local_node_name(name).await;
        }
        let groups = args.groups.clone();
        if !groups.is_empty() {
            discovery.set_local_groups(groups.clone()).await;
        }

        let ollama = OllamaManager::new(args.ollama_base_url.clone()).await;

        // Load models once at startup.
        let models = match args.models {
            Some(models) => models,
            None => match ollama.list_models().await {
                Ok(list) => {
                    let mut out: Vec<String> = list.into_iter().map(|m| m.name).collect();
                    out.sort();
                    out.dedup();
                    out
                }
                Err(e) => {
                    tracing::warn!("Failed to list local Ollama models: {e}");
                    Vec::new()
                }
            },
        };

        discovery
            .set_ollama_offer(true, models.clone(), args.ollama_base_url.clone())
            .await;
        discovery.broadcast_presence();

        tracing::info!(
            "vas-ollama-share started. groups={:?} models={:?} base_url={:?}",
            groups,
            models,
            args.ollama_base_url
        );

        // Keep announcing periodically via NetworkDiscovery background loop.
        // We also refresh debug stats so users can see progress in logs.
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

#[derive(Debug, Default, Clone)]
struct Args {
    name: Option<String>,
    groups: Vec<String>,
    models: Option<Vec<String>>,
    ollama_base_url: Option<String>,
}

impl Args {
    fn from_env() -> Self {
        // Minimal parsing:
        //   --name <name>
        //   --group <groupId>   (repeatable)
        //   --groups <csv>
        //   --models <csv>
        //   --ollama <base_url>
        let mut out = Args::default();
        let mut it = std::env::args().skip(1);
        while let Some(a) = it.next() {
            match a.as_str() {
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
                        out.groups.extend(split_csv(&csv));
                    }
                }
                "--models" => {
                    if let Some(csv) = it.next() {
                        out.models = Some(split_csv(&csv));
                    }
                }
                "--ollama" => out.ollama_base_url = it.next(),
                _ => {}
            }
        }

        out
    }
}

fn split_csv(s: &str) -> Vec<String> {
    s.split(',')
        .map(|x| x.trim())
        .filter(|x| !x.is_empty())
        .map(|x| x.to_string())
        .collect()
}
