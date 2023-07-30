use clap::{Parser, Subcommand};
use std::path::PathBuf;

pub struct Config {
    pub command: Command,
    pub file_path: std::path::PathBuf,
    pub git_path: std::path::PathBuf,
    pub commit_id: Option<String>,
}

impl Config {
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
    /// Optional path to the yaml config file [default: ".mkchlog.yml"]
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

#[derive(Subcommand)]
pub enum Command {
    /// Verify the structure of commit messages
    Check,
    /// Process git history and output the changelog in markdown format
    #[command(name = "gen")]
    Generate,
}
