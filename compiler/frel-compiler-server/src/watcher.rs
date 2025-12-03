// File watching using notify crate

use std::path::Path;
use std::sync::mpsc;
use std::time::Duration;

use anyhow::Result;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};

use crate::compiler;
use crate::state::SharedState;

/// Run the file watcher
pub async fn run_watcher(state: SharedState, root: impl AsRef<Path>) -> Result<()> {
    let root = root.as_ref().to_path_buf();

    // Create a channel for file system events
    let (tx, rx) = mpsc::channel();

    // Create the watcher
    let mut watcher = RecommendedWatcher::new(
        move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        },
        Config::default().with_poll_interval(Duration::from_millis(100)),
    )?;

    // Watch the project directory recursively
    watcher.watch(&root, RecursiveMode::Recursive)?;

    println!("Watching for changes in {}", root.display());

    // Process events
    loop {
        // Use recv_timeout to allow periodic checks
        match rx.recv_timeout(Duration::from_millis(500)) {
            Ok(event) => {
                // Filter for .frel files and clone them
                let frel_paths: Vec<_> = event
                    .paths
                    .iter()
                    .filter(|p| p.extension().map(|e| e == "frel").unwrap_or(false))
                    .cloned()
                    .collect();

                if frel_paths.is_empty() {
                    continue;
                }

                // Debounce: collect all events for a short period
                let mut all_paths = frel_paths;
                while let Ok(more_event) = rx.recv_timeout(Duration::from_millis(50)) {
                    let more_frel: Vec<_> = more_event
                        .paths
                        .iter()
                        .filter(|p| p.extension().map(|e| e == "frel").unwrap_or(false))
                        .cloned()
                        .collect();
                    all_paths.extend(more_frel);
                }

                // Deduplicate paths
                let mut unique_paths: Vec<_> = all_paths;
                unique_paths.sort();
                unique_paths.dedup();

                // Process each changed file
                for path in unique_paths {
                    println!("File changed: {}", path.display());

                    let result = {
                        let mut state = state.write().await;
                        compiler::handle_file_change(&mut state, &path)
                    };

                    if !result.modules_rebuilt.is_empty() {
                        println!(
                            "  Rebuilt {} module(s) in {:?}, {} error(s)",
                            result.modules_rebuilt.len(),
                            result.duration,
                            result.error_count
                        );
                    }
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                // No events, continue
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                // Channel closed, exit
                break;
            }
        }
    }

    Ok(())
}
