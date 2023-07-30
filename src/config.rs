//! Configuration of the CLI application
//!
//! Determines how to run the application, what command to dispatch etc. by parsing the command line arguments

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Configuration object for the application
pub struct Config {
    /// The name of the called command
    pub command: Command,
    /// Path to the config (template) file
    pub file_path: std::path::PathBuf,
    /// Path to the git repository
    pub git_path: std::path::PathBuf,
    /// Commit number to start. Previous commits will be skipped during processing.
    pub commit_id: Option<String>,
}

impl Config {
    /// Creates and initializes a new config.
    pub fn new() -> Result<Self, String> {
        let args = Args::parse();

        Ok(Self {
            command: args.command,
            file_path: args.file_path.unwrap_or(PathBuf::from(".mkchlog.yml")),
            git_path: args.git_path.unwrap_or(PathBuf::from("./")),
            commit_id: args.commit,
        })
    }
}

/// Application arguments
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Optional path to the YAML template file [default: ".mkchlog.yml"]
    #[arg(short, long)]
    file_path: Option<PathBuf>,

    /// Optional path to the git repository (path to the .git directory) [default: "./"]
    #[arg(short, long)]
    git_path: Option<PathBuf>,

    /// Optional commit number. Previous commits will be skipped. By default, all commit messages are checked.
    #[arg(short, long)]
    commit: Option<String>,

    #[command(subcommand)]
    command: Command,
}

/// Application commands
#[derive(Subcommand)]
pub enum Command {
    /// Verify the structure of commit messages
    Check,
    /// Process git history and output the changelog in markdown format
    #[command(name = "gen")]
    Generate,
}
