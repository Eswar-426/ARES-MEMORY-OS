use ares_core::ProjectId;
use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode};
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

pub struct ProjectWatcher {
    _project_id: ProjectId,
    root_path: PathBuf,
}

impl ProjectWatcher {
    pub fn new(project_id: ProjectId, root_path: PathBuf) -> Self {
        Self { _project_id: project_id, root_path }
    }

    /// Watch the directory for changes, returning a channel of batched file paths that changed.
    pub fn watch(self) -> Result<mpsc::Receiver<Vec<PathBuf>>, Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = mpsc::channel();
        
        std::thread::spawn(move || {
            let (debounce_tx, debounce_rx) = mpsc::channel();
            let mut debouncer = match new_debouncer(Duration::from_millis(500), debounce_tx) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("Failed to create debouncer: {}", e);
                    return;
                }
            };
            
            if let Err(e) = debouncer.watcher().watch(&self.root_path, RecursiveMode::Recursive) {
                eprintln!("Failed to watch {}: {}", self.root_path.display(), e);
                return;
            }

            for result in debounce_rx {
                match result {
                    Ok(events) => {
                        let paths: Vec<PathBuf> = events.into_iter().map(|e| e.path).collect();
                        if tx.send(paths).is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("Watcher error: {:?}", e);
                    }
                }
            }
        });

        Ok(rx)
    }
}
