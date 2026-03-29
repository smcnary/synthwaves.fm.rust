use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::time::{Duration, sleep};
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecurringJob {
    pub name: String,
    pub interval_seconds: u64,
}

pub async fn run_scheduler(jobs: Vec<RecurringJob>) {
    loop {
        let started_at: DateTime<Utc> = Utc::now();
        for job in &jobs {
            info!(job = %job.name, "tick recurring job");
        }
        let elapsed = (Utc::now() - started_at).num_milliseconds().max(0) as u64;
        let sleep_ms = 1000_u64.saturating_sub(elapsed);
        sleep(Duration::from_millis(sleep_ms)).await;
    }
}
