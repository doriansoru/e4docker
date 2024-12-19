use std::{error, process::Command};

/// A struct which holds a [Command] and its arguments.
pub struct E4Command {
    cmd: String,
    arguments: String,
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
    ///     String::from("/tmp/myfile.txt"));
    /// ```
    pub fn new(cmd: String, arguments: String) -> Self {
        Self { cmd, arguments }
    }

    /// Exec the [Command] of the [E4Command]. Return () or the [error::Error].
    pub fn exec(&mut self) -> Result<(), Box<dyn error::Error>> {
        // With arguments
        if !self.arguments.is_empty() {
            let _ = Command::new(self.cmd.as_str())
                .args([self.arguments.as_str()])
                .spawn()?;
        } else {
            let _ = Command::new(self.cmd.as_str())
                .spawn()?;
        }
        Ok(())
    }

    /// Set the [E4Command] and its args.
    pub fn set(&mut self, cmd: String, arguments: String) {
        self.cmd = cmd;
        self.arguments = arguments;
    }

    /// Get the args of the [E4Command].
    pub fn get_arguments(&self) -> &String {
        &self.arguments
    }


    /// Get the [Command] of the [E4Command].
    pub fn get_cmd(&self) -> &String {
        &self.cmd
    }
}
