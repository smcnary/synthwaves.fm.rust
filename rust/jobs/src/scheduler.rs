use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::time::{Duration, sleep};
use tracing::{error, info};

use crate::workers;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecurringJob {
    pub name: String,
    pub interval_seconds: u64,
}

pub async fn run_scheduler(jobs: Vec<RecurringJob>) {
    let mut last_run: HashMap<String, DateTime<Utc>> = HashMap::new();
    loop {
        let started_at: DateTime<Utc> = Utc::now();
        for job in &jobs {
            let should_run = last_run
                .get(&job.name)
                .map(|at| (Utc::now() - *at).num_seconds() >= job.interval_seconds as i64)
                .unwrap_or(true);
            if !should_run {
                continue;
            }
            info!(job = %job.name, "run recurring job");
            let result = match job.name.as_str() {
                "station_listener_sync" => workers::station_listener_sync_job().await,
                "youtube_playlist_sync" => workers::youtube_playlist_sync_job().await,
                "database_backup" => workers::database_backup_job().await,
                _ => {
                    info!(job = %job.name, "unknown recurring job name; skipping");
                    Ok(())
                }
            };
            if let Err(err) = result {
                error!(job = %job.name, error = %err, "recurring job failed");
            } else {
                last_run.insert(job.name.clone(), Utc::now());
            }
        }
        let elapsed = (Utc::now() - started_at).num_milliseconds().max(0) as u64;
        let sleep_ms = 1000_u64.saturating_sub(elapsed);
        sleep(Duration::from_millis(sleep_ms)).await;
    }
}
