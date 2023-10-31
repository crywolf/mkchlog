//! Implements reading from standard input instead of using `git log` command.  Useful to use in a commit-msg git hook.

use std::io::{stdin, Read};

/// Reads from standard input instead of using `git log` command.
pub struct Stdin {}

impl Stdin {
    /// Creates a new [`Stdin`] that reads commit(s) from standard input.
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for Stdin {
    fn default() -> Self {
        Self::new()
    }
}

impl super::GitLogCommand for Stdin {
    fn get_log(&self) -> Result<String, Box<dyn std::error::Error>> {
        let mut stdin = stdin().lock();
        let mut buf = String::new();
        stdin.read_to_string(&mut buf)?;

        if !buf.starts_with("commit ") {
            // prepend fake header to make it look like valid commit
            // when reading not-yet-commited commit
            buf.insert_str(0, "commit FROM STDIN\n\n")
        }

        Ok(buf)
    }
}
