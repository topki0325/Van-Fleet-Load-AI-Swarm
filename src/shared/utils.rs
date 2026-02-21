use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub fn compute_hash(data: &[u8]) -> String {
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

pub fn generate_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

pub fn format_duration(duration: std::time::Duration) -> String {
    let secs = duration.as_secs();
    let mins = secs / 60;
    let hours = mins / 60;
    format!("{}h {}m {}s", hours, mins % 60, secs % 60)
}

pub fn parse_timestamp(ts: &str) -> Result<chrono::DateTime<chrono::Utc>, Box<dyn std::error::Error>> {
    Ok(chrono::DateTime::parse_from_rfc3339(ts)?.with_timezone(&chrono::Utc))
}