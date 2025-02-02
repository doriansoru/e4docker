use std::{error, thread, process::Command, sync::{Arc, Mutex}};
use crate::{tr, translations::Translations};

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
    pub fn exec(&mut self, translations: Arc<Mutex<Translations>>) -> Result<(), Box<dyn error::Error>> {
        // With arguments
        let cmd = self.cmd.clone();
        let args = self.arguments.clone();
        let translations_clone = translations.clone();
        if !self.arguments.is_empty() {
            thread::spawn(move || {
                let child = Command::new(&cmd)
                    .spawn();
                match child {
                    Ok(mut c) => {
                        let _ = c.wait(); // Wait nel thread separato
                    },
                    Err(e) => {
                        let message = tr!(
                            translations_clone,
                            format,
                            "failed-to-execute-command",
                            &[&cmd, &e.to_string()]
                        );
                        fltk::dialog::alert_default(&message);
                    }
                }
            });
        } else {
            thread::spawn(move || {
                let child = Command::new(&cmd)
                    .args([&args])
                    .spawn();
                match child {
                    Ok(mut c) => {
                        let _ = c.wait(); // Wait nel thread separato
                    },
                    Err(e) => {
                        let message = tr!(
                            translations_clone,
                            format,
                            "failed-to-execute-command",
                            &[&cmd, &e.to_string()]
                        );
                        fltk::dialog::alert_default(&message);
                    }
                }
            });
        }
        Ok(())
    }

    /// Get the [E4Command]
    pub fn get(&self) -> &String {
        &self.cmd
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
