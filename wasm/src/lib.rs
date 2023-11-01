use std::str::FromStr;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn check(config: &str, git_callback: js_sys::Function) -> Result<(), JsValue> {
    run(config, git_callback).map_err(|error| {
        use std::fmt::Write;

        let mut error_message = format!("Error: {}", error);
        let mut source = error.source();
        while let Some(error) = source {
            write!(error_message, ": {}", error).expect("writing to string never fails");
            source = error.source();
        }
        error_message.into()
    })
}

fn run(config: &str, git_callback: js_sys::Function) -> Result<(), Box<dyn std::error::Error>> {
    use mkchlog::changelog::Changelog;
    use mkchlog::changelog::Changes;
    use mkchlog::template::Template;

    let mut template = Template::<Changes>::from_str(config)?;

    let git_cmd = GitCmd {
        callback: git_callback,
        commit_id: template.settings.skip_commits_up_to.clone(),
    };
    let git_cmd = Box::new(git_cmd);
    let git = mkchlog::git::Git::new(git_cmd);

    let mut changelog = Changelog::new(&mut template, git);

    changelog.generate()?;
    Ok(())
}

struct GitCmd {
    commit_id: Option<String>,
    callback: js_sys::Function,
}

impl mkchlog::git::GitLogCommand for GitCmd {
    fn get_log(&self) -> Result<String, Box<dyn std::error::Error>> {
        let args = js_sys::Array::new();
        args.push(&JsValue::from("log"));
        args.push(&JsValue::from("--no-merges"));
        if let Some(commit_id) = &self.commit_id {
            args.push(&JsValue::from(format!("{}..HEAD", commit_id)));
        }
        let args = JsValue::from(args);
        let result = self
            .callback
            .call1(&JsValue::null(), &args)
            .map_err(|error| format!("{:?}", error))?;
        result
            .as_string()
            .ok_or_else(|| "The value returned by closure is not a string".into())
    }
}
