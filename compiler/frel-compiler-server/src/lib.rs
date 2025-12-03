// Frel Compiler Server
//
// An always-compiled daemon that watches a project directory,
// compiles Frel source files on change, and provides compilation
// results via HTTP API.

pub mod api;
pub mod compiler;
pub mod events;
pub mod server;
pub mod state;
pub mod watcher;

pub use events::CompilationEvent;
pub use state::{ProjectState, SharedState};
