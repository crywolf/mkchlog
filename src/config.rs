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
    pub git_path: Option<std::path::PathBuf>,
    /// Commit number to start. This one and previous commits will be skipped during processing.
    pub commit_id: Option<String>,
    /// Name of the project in multi-project repository for which we want to generate changelog.
    pub project: Option<String>,
    /// Read commit(s) from stdin
    pub read_from_stdin: bool,
}

impl Config {
    /// Creates and initializes a new config.
    pub fn new() -> Result<Self, String> {
        let args = Args::parse();

        Ok(Self {
            command: args.command,
            file_path: args.file_path.unwrap_or(PathBuf::from(".mkchlog.yml")),
            git_path: args.git_path,
            commit_id: args.commit,
            project: args.project,
            read_from_stdin: args.from_stdin,
        })
    }
}

/// Application arguments
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the project in multi-project repository for which we want to generate changelog.
    #[arg(short, long)]
    project: Option<String>,

    /// Optional commit number. This one and previous commits will be skipped. By default, all commit messages are checked.
    #[arg(short, long)]
    commit: Option<String>,

    /// Optional path to the YAML template file [default: ".mkchlog.yml"]
    #[arg(short, long)]
    file_path: Option<PathBuf>,

    /// Optional path to the git repository (path to the .git directory) [default: "./"]
    #[arg(short, long)]
    git_path: Option<PathBuf>,

    /// Read commit(s) from stdin
    #[arg(long, default_value_t = false)]
    from_stdin: bool,

    #[command(subcommand)]
    command: Command,
}

/// Application commands
#[derive(Subcommand, PartialEq)]
pub enum Command {
    /// Verify the structure of commit messages
    Check,
    /// Process git history and output the changelog in markdown format
    #[command(name = "gen")]
    Generate,
}
