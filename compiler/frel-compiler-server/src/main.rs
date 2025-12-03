// Frel Compiler Server CLI
//
// Command-line interface for the Frel compiler server.

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use clap::Parser;
use tokio::sync::{watch, RwLock};

use frel_compiler_server::state::ProjectState;
use frel_compiler_server::{compiler, server, watcher};

#[derive(Parser)]
#[command(name = "frel-server")]
#[command(about = "Frel compiler server - always-compiled daemon", long_about = None)]
#[command(version)]
struct Cli {
    /// Project directory
    #[arg(default_value = ".")]
    project: PathBuf,

    /// HTTP port
    #[arg(short, long, default_value = "3001")]
    port: u16,

    /// Build output directory
    #[arg(short, long, default_value = "build")]
    output: PathBuf,

    /// Exit after first compilation (for CI/scripts)
    #[arg(long)]
    once: bool,
}

#[actix_web::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Resolve paths
    let project_root = cli.project.canonicalize().unwrap_or(cli.project.clone());
    let build_dir = if cli.output.is_absolute() {
        cli.output.clone()
    } else {
        project_root.join(&cli.output)
    };

    println!("Frel Compiler Server");
    println!("  Project: {}", project_root.display());
    println!("  Output:  {}", build_dir.display());
    println!();

    // Create shared state
    let state = Arc::new(RwLock::new(ProjectState::new(
        project_root.clone(),
        build_dir,
    )));

    // Initial compilation
    println!("Building project...");
    let build_result = {
        let mut state = state.write().await;
        compiler::full_build(&mut state)
    };

    println!(
        "Build completed in {:?}: {} module(s), {} error(s)",
        build_result.duration, build_result.modules_built, build_result.error_count
    );

    if cli.once {
        // Exit after first compilation
        std::process::exit(if build_result.error_count > 0 { 1 } else { 0 });
    }

    // Create shutdown channel for coordinating graceful shutdown
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    // Start file watcher
    let watcher_state = state.clone();
    let watcher_root = project_root.clone();
    let watcher_handle = actix_rt::spawn(async move {
        if let Err(e) = watcher::run_watcher(watcher_state, watcher_root, shutdown_rx).await {
            eprintln!("File watcher error: {}", e);
        }
    });

    // Start HTTP server
    println!();
    println!("Server listening on http://localhost:{}", cli.port);
    println!("Press Ctrl-C to stop");

    // Create the server but don't await it yet
    let server = server::run_server(state, cli.port)?;
    let server_handle = server.handle();

    // Spawn task to handle shutdown signals (Ctrl-C and SIGTERM)
    let signal_handle = server_handle.clone();
    actix_rt::spawn(async move {
        // Wait for shutdown signal
        let shutdown_reason = tokio::select! {
            _ = tokio::signal::ctrl_c() => "Ctrl-C",
            _ = async {
                #[cfg(unix)]
                {
                    use tokio::signal::unix::{signal, SignalKind};
                    let mut sigterm = signal(SignalKind::terminate()).expect("Failed to register SIGTERM handler");
                    sigterm.recv().await;
                }
                #[cfg(not(unix))]
                {
                    // On non-Unix systems, just wait forever (Ctrl-C will work)
                    std::future::pending::<()>().await;
                }
            } => "SIGTERM",
        };

        println!();
        println!("Received {}, shutting down...", shutdown_reason);

        // Signal the file watcher to stop
        let _ = shutdown_tx.send(true);

        // Stop the HTTP server gracefully
        signal_handle.stop(true).await;
    });

    // Run the server until it's stopped
    server.await?;

    // Wait for watcher to finish
    let _ = watcher_handle.await;

    println!("Server stopped");

    Ok(())
}
