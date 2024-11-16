use std::{error, process::Command};

/// A struct which holds a [Command] and its arguments.
pub struct E4Command {
    cmd: String,
    args: Vec<String>,
}

impl E4Command {
    /// Create a new E4Command.
    ///
    /// # Example
    ///
    /// Create a [E4Command] to start /usr/bin/nano /tmp/myfile.txt.
    ///
    /// ```rust
    /// use e4docker::e4command::E4Command;
    ///
    /// let command = E4Command::new(
    ///     String::from("/usr/bin/nano"),
    ///     vec![String::from("/tmp/myfile.txt")]);
    /// ```
    pub fn new(cmd: String, args: Vec<String>) -> Self {
        Self { cmd, args }
    }

    /// Exec the [Command] of the [E4Command]. Return () or the [error::Error].
    pub fn exec(&self) -> Result<(), Box<dyn error::Error>> {
        let _ = Command::new(&self.cmd)
            .args(&self.args)
            .spawn()?;
        Ok(())
    }

    /// Set the [E4Command] and its args.
    pub fn set(&mut self, cmd: String, args: Vec<String>) {
        self.cmd = cmd;
        self.args = args;
    }

    /// Get the [Command] of the [E4Command].
    pub fn get_cmd(&self) -> &String {
        &self.cmd
    }

    /// Get the args of the [E4Command].
    pub fn get_args(&self) -> &Vec<String> {
        &self.args
    }
}
