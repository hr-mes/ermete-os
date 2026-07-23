use std::time::SystemTime;
use tokio::fs;

pub struct CoredumpWatcher {
    last_checked: SystemTime,
}

impl CoredumpWatcher {
    pub fn new() -> Self {
        Self {
            last_checked: SystemTime::now(),
        }
    }

    pub async fn poll_new_crashes(&mut self) -> Option<String> {
        let path = "/var/lib/systemd/coredump/";
        let mut newest_crash = None;
        let mut newest_time = self.last_checked;

        if let Ok(mut entries) = fs::read_dir(path).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                if let Ok(metadata) = entry.metadata().await {
                    if metadata.is_file() {
                        if let Ok(modified) = metadata.modified() {
                            if modified > newest_time {
                                newest_time = modified;
                                newest_crash = Some(entry.file_name().to_string_lossy().to_string());
                            }
                        }
                    }
                }
            }
        }

        if newest_crash.is_some() {
            self.last_checked = newest_time;
        }

        newest_crash
    }
}
