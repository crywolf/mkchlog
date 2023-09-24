use mkchlog::git::GitLogCommand;
use std::error::Error;

pub struct GitCmdMock {
    log: String,
}

impl GitCmdMock {
    pub fn new(log: String) -> Self {
        Self { log }
    }
}

impl GitLogCommand for GitCmdMock {
    fn get_log(&self) -> Result<String, Box<dyn Error>> {
        Ok(self.log.to_string())
    }
}
