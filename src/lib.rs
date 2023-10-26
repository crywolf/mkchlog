//! Changelog generator tool suitable for user-facing changelogs and based on experiences of existing projects.
//!
//! Refer to `README.md` for more information

pub mod changelog;
pub mod config;
pub mod git;
pub mod template;

use crate::changelog::Changelog;
use crate::config::Command;
use crate::git::command::GitLogCmd;
use crate::git::Git;
use crate::template::Template;
use std::fs::File;

/// Entrypoint of the application
pub fn run(config: config::Config) -> Result<(), Box<dyn std::error::Error>> {
    match &config.command {
        Command::Generate => println!("{}", generate(config)?),
        Command::Check => { generate(config)?; },
        Command::InitGhAction => init_gh_action(config)?,
    }

    Ok(())
}

fn generate(config: config::Config) -> Result<String, Box<dyn std::error::Error>> {
    let f = match File::open(&config.file_path) {
        Ok(f) => f,
        Err(err) => {
            return Err(format!(
                "Error reading config YAML file '{}': {}",
                config.file_path.display(),
                err
            )
            .into())
        }
    };

    let template = Template::new(f)?;

    // set value from program arguments or yaml file
    let commit_id = match (
        config.commit_id,
        template.settings.skip_commits_up_to.as_ref(),
    ) {
        (Some(commit_id), _) => Some(commit_id),
        (None, Some(commit_id)) => Some(commit_id.to_owned()),
        (None, None) => None,
    };

    // set value from program arguments or yaml file
    let git_path = match (config.git_path, template.settings.git_path.as_ref()) {
        (Some(git_path), _) => git_path,
        (_, Some(git_path)) => git_path.to_owned(),
        (None, None) => std::path::PathBuf::from("./"),
    };

    let git_cmd = Box::new(GitLogCmd::new(git_path, commit_id));
    let git = Git::new(git_cmd);

    let changelog = Changelog::new(template, git);

    changelog.produce()
}

fn init_gh_action(config: config::Config) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Write;

    let git_path = match config.git_path {
        Some(git_path) => git_path,
        None => {
            // TODO: windows support
            use std::os::unix::ffi::OsStringExt;

            let mut output = std::process::Command::new("git")
                .arg("rev-parse")
                .arg("--show-toplevel")
                // output() would capture stderr without this
                .stderr(std::process::Stdio::inherit())
                .output()
                .map_err(|error| format!("Failed to run git rev-parse --toplevel: {}", error))?;
            if !output.status.success() {
                return Err(format!("git rev-parse --toplevel failed with exit status {}", output.status).into())
            }
            if output.stdout.ends_with(b"\n") {
                output.stdout.pop();
            }
            std::ffi::OsString::from_vec(output.stdout).into()
        },
    };

    let workflows_dir = git_path.join(".github/workflows");
    std::fs::create_dir_all(&workflows_dir)
        .map_err(|error| format!("Failed to create directory {}: {}", workflows_dir.display(), error))?;
    let action_yml_path = workflows_dir.join("changelog.yml");
    let mut action_yml = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .create_new(true)
        .open(&action_yml_path)
        .map_err(|error| format!("Failed to open {}: {}", action_yml_path.display(), error))?;
    action_yml.write_all(STANDARD_GH_ACTION)
        .map_err(|error| format!("Failed to write to {}: {}", action_yml_path.display(), error))?;
    // flushing not needed on bare File

    // Currently hardcoded because GH action doesn't support parameters yet
    let template_path = git_path.join(".mkchlog.yml");
    let template_yml = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .create_new(true)
        .open(&template_path);

    let template_yml = match template_yml {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            // We consider this not an error - someone could migrate to GitHub.
            eprintln!("Note: {} not created because it already exists", template_path.display());
            return Ok(());
        },
        Err(error) => return Err(format!("Failed to open {}: {}", template_path.display(), error).into()),
    };
    let mut template_yml = std::io::BufWriter::new(template_yml);
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(&git_path)
        .arg("rev-parse")
        .arg("HEAD")
        .output()
        .map_err(|error| format!("Failed to run git rev-parse HEAD: {}", error))?;

    if output.status.success() {
        let commit = output.stdout;

        (|| -> Result<_, std::io::Error> {
            template_yml.write_all(b"# mkchlog was introduced after this commit\nskip-commits-up-to: ")?;
            template_yml.write_all(&commit)?;
            template_yml.write_all(b"\n")?;

            Ok(())
        })().map_err(|error| format!("Failed to write to {}: {}", template_path.display(), error))?;
    } else {
        // When the repo has no commits git rev-parse HEAD returns HEAD
        // So there's not skip-commits-up-to
        if output.stdout != b"HEAD\n" {
            // If stderr is broken we can't do much, so just ignore it.
            let _ = std::io::stderr().lock().write_all(&output.stderr);
            return Err(format!("git rev-parse HEAD failed with exit status {}", output.status).into())
        }
    }

    (|| -> Result<_, std::io::Error> {
        template_yml.write_all(&STANDARD_MKCHLOG_YML)?;
        template_yml.flush()?;
        Ok(())
    })()
        .map_err(|error| format!("Failed to write to {}: {}", template_path.display(), error))?;

    Ok(())
}

static STANDARD_GH_ACTION: &[u8] = b"
on:
  push:
    branches:
      - master
      - 'test-ci/**'
  pull_request:

name: Check changelog

jobs:
  changelog:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout repository
        uses: actions/checkout@v2
        with:
          # This is critically important!
          # The action won't work without this
          fetch-depth: 0
      - name: Check commits
        uses: Kixunil/mkchlog-action@master
";

static STANDARD_MKCHLOG_YML: &[u8] = b"
sections:
    security:
        title: Security
        description: This section contains very important security-related changes.
        subsections:
            vuln_fixes:
                title: Fixed vulnerabilities
    breaking:
        title: Breaking changes
    feat:
        title: New features
    bug:
        title: Fixed bugs
    perf:
        title: Performance improvements
    doc:
        title: Documentation changes
    dev:
        title: Development
        description: Internal development changes
";
