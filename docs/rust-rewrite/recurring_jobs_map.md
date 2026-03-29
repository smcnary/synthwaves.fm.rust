# Recurring Jobs Mapping

Source of truth in Rails: `config/recurring.yml`

## Mapped recurring jobs

| Rails Schedule Item | Rust Mapping |
| --- | --- |
| `clear_solid_queue_finished_jobs` | scheduler task placeholder |
| `cleanup_expired_downloads` | scheduler task placeholder |
| `cleanup_stale_downloads` | scheduler task placeholder |
| `iptv_channel_sync` | scheduler task placeholder |
| `epg_sync` | scheduler task placeholder |
| `epg_cleanup` | scheduler task placeholder |
| `database_backup` | `jobs::workers::database_backup_job` |
| `station_listener_sync` | `jobs::workers::station_listener_sync_job` |

## Runtime model

- `jobs::scheduler::RecurringJob` is the canonical recurring task type.
- Worker implementations in `jobs::workers` are async and callable from scheduler ticks.
- ffmpeg-backed jobs are implemented for audio/video conversion with non-interactive `-y` overwrite behavior.
