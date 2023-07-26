use std::error::Error;

pub struct GitLogCmd {
    path: String,
}

impl GitLogCmd {
    pub fn new(path: String) -> Self {
        Self { path }
    }
}

impl super::GitLogCommand for GitLogCmd {
    fn get_log(&self) -> Result<String, Box<dyn Error>> {
        let mut git_command = std::process::Command::new("git");
        git_command.arg("-C").arg(&self.path).arg("log");

        let git_cmd_output = git_command.output().map_err(|err| {
            format!(
                "Failed to execute '{}' command: {}",
                git_command.get_program().to_str().unwrap_or("git"),
                err
            )
        })?;

        if !git_cmd_output.status.success() {
            let args: Vec<_> = git_command
                .get_args()
                .map(|a| a.to_str().unwrap_or("git log"))
                .collect();

            return Err(format!(
                "Failed to execute 'git {}' command:\n{}",
                args.join(" "),
                String::from_utf8_lossy(&git_cmd_output.stderr).into_owned()
            )
            .into());
        }

        let git_log = String::from_utf8_lossy(&git_cmd_output.stdout);

        Ok(git_log.into_owned())
    }
}
