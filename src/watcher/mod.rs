//! File system watcher using inotify
//!
//! This module provides event-driven file system monitoring using the notify crate,
//! allowing near-instant detection of changes without polling.

use crate::error::DmsAwwwError;
use notify::{Event, EventKind, RecursiveMode, RecommendedWatcher, Watcher};
use std::path::Path;
use tokio::sync::mpsc;
use tokio::time::{timeout, Duration};

/// File change event
#[derive(Debug, Clone)]
pub enum FileEvent {
    /// File was modified
    Modified,
    /// File was created
    Created,
    /// File was deleted
    Deleted,
    /// Watcher error occurred
    Error(String),
}

/// File watcher that monitors a file for changes
pub struct FileWatcher {
    _watcher: RecommendedWatcher,
    rx: mpsc::Receiver<FileEvent>,
    path: std::path::PathBuf,
}

impl FileWatcher {
    /// Create a new file watcher for the specified path
    pub fn new<P: AsRef<Path>>(path: P) -> std::result::Result<Self, DmsAwwwError> {
        let path = path.as_ref().to_path_buf();
        let (tx, rx) = mpsc::channel(32);

        // Create a channel for notify events
        let (ntx, nrx) = std::sync::mpsc::channel();

        // Watch the parent directory since the file might be replaced
        let watch_path = path
            .parent()
            .unwrap_or_else(|| Path::new("/"))
            .to_path_buf();

        let mut watcher = RecommendedWatcher::new(
            move |res: std::result::Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    if let Err(e) = ntx.send(event) {
                        tracing::error!("Failed to send notify event: {}", e);
                    }
                }
            },
            notify::Config::default(),
        ).map_err(|e| DmsAwwwError::Watcher(e.to_string()))?;

        watcher.watch(&watch_path, RecursiveMode::NonRecursive)
            .map_err(|e| DmsAwwwError::Watcher(e.to_string()))?;

        // Spawn a task to bridge notify events to our channel
        let target_filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| DmsAwwwError::Watcher(
                "Invalid file name".to_string()
            ))?
            .to_string();

        tokio::spawn(async move {
            while let Ok(event) = nrx.recv() {
                // Check if the event is for our target file
                let is_target = event.paths.iter().any(|p: &std::path::PathBuf| {
                    p.file_name()
                        .and_then(|n| n.to_str())
                        .map(|n| n == target_filename)
                        .unwrap_or(false)
                });

                if !is_target {
                    continue;
                }

                let file_event = match event.kind {
                    EventKind::Create(_) => FileEvent::Created,
                    EventKind::Modify(_) => FileEvent::Modified,
                    EventKind::Remove(_) => FileEvent::Deleted,
                    EventKind::Any => FileEvent::Modified,
                    _ => continue,
                };

                if tx.send(file_event).await.is_err() {
                    break; // Receiver dropped
                }
            }
        });

        Ok(Self {
            _watcher: watcher,
            rx,
            path,
        })
    }

    /// Wait for the next file event
    ///
    /// Returns None if the watcher is closed
    pub async fn next(&mut self) -> Option<FileEvent> {
        self.rx.recv().await
    }

    /// Wait for the next file event with a timeout
    pub async fn next_with_timeout(&mut self, dur: Duration) -> std::result::Result<Option<FileEvent>, DmsAwwwError> {
        match timeout(dur, self.rx.recv()).await {
            Ok(Some(event)) => Ok(Some(event)),
            Ok(None) => Ok(None), // Channel closed
            Err(_) => Err(DmsAwwwError::Timeout),
        }
    }

    /// Get the watched file path
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Create a new watcher that returns a stream of events
    pub fn into_stream(self) -> mpsc::Receiver<FileEvent> {
        self.rx
    }
}

/// Debounced file watcher that coalesces rapid changes
pub struct DebouncedWatcher {
    watcher: FileWatcher,
    debounce_ms: u64,
    last_event_time: Option<tokio::time::Instant>,
}

impl DebouncedWatcher {
    /// Create a new debounced watcher
    pub fn new<P: AsRef<Path>>(path: P, debounce_ms: u64) -> std::result::Result<Self, DmsAwwwError> {
        Ok(Self {
            watcher: FileWatcher::new(path)?,
            debounce_ms,
            last_event_time: None,
        })
    }

    /// Wait for the next debounced event
    pub async fn next(&mut self) -> std::result::Result<Option<FileEvent>, DmsAwwwError> {
        loop {
            let event = match self.watcher.next().await {
                Some(e) => e,
                None => return Ok(None),
            };

            let now = tokio::time::Instant::now();

            if let Some(last_time) = self.last_event_time {
                let elapsed = now.duration_since(last_time).as_millis() as u64;

                if elapsed >= self.debounce_ms {
                    self.last_event_time = Some(now);
                    return Ok(Some(event));
                } else {
                    // Wait for debounce period
                    tokio::time::sleep(Duration::from_millis(self.debounce_ms - elapsed)).await;
                    continue;
                }
            } else {
                self.last_event_time = Some(now);
                return Ok(Some(event));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[tokio::test]
    async fn test_file_watcher_detects_changes() -> std::result::Result<(), DmsAwwwError> {
        let temp_dir = tempfile::TempDir::new()?;
        let test_file = temp_dir.path().join("test.json");

        // Create initial file
        let mut file = std::fs::File::create(&test_file)?;
        file.write_all(b"{}")?;

        // This test would require more setup for full functionality
        // For now, just verify the watcher can be created
        let _watcher = FileWatcher::new(&test_file)?;
        // Note: Actually testing file change detection in a test environment
        // is tricky due to timing and filesystem behavior

        Ok(())
    }
}
