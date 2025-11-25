// Frel CLI Tool
//
// Command-line interface for the Frel compiler.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "frel")]
#[command(about = "Frel language compiler", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile a Frel source file
    Compile {
        /// Input Frel file
        #[arg(value_name = "FILE")]
        input: PathBuf,

        /// Output file (defaults to input with .js extension)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Target language (currently only 'javascript')
        #[arg(short, long, default_value = "javascript")]
        target: String,
    },

    /// Check a Frel file for errors without compiling
    Check {
        /// Input Frel file
        #[arg(value_name = "FILE")]
        input: PathBuf,
    },

    /// Show version information
    Version,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Compile {
            input,
            output,
            target,
        } => compile(&input, output.as_deref(), &target),
        Commands::Check { input } => check(&input),
        Commands::Version => {
            println!("frelc {}", env!("CARGO_PKG_VERSION"));
            println!("frel-compiler-core {}", frel_compiler_core::VERSION);
            Ok(())
        }
    }
}

fn compile(input: &Path, output: Option<&Path>, target: &str) -> Result<()> {
    // Read input file
    let source = fs::read_to_string(input)
        .with_context(|| format!("Failed to read input file: {}", input.display()))?;

    // Parse and compile
    let ast = frel_compiler_core::compile(&source)
        .with_context(|| "Compilation failed")?;

    // Generate code
    let code = match target {
        "javascript" | "js" => frel_compiler_plugin_javascript::generate(&ast),
        _ => anyhow::bail!("Unsupported target: {}", target),
    };

    // Determine output path
    let output_path = output
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| input.with_extension("js"));

    // Write output
    fs::write(&output_path, code)
        .with_context(|| format!("Failed to write output file: {}", output_path.display()))?;

    println!("Compiled {} -> {}", input.display(), output_path.display());

    Ok(())
}

fn check(input: &Path) -> Result<()> {
    // Read input file
    let source = fs::read_to_string(input)
        .with_context(|| format!("Failed to read input file: {}", input.display()))?;

    // Parse and check
    frel_compiler_core::compile(&source)
        .with_context(|| "Check failed")?;

    println!("✓ {} OK", input.display());

    Ok(())
}
