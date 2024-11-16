use configparser::ini::Ini;
use fltk::{button::Button, draw, enums::ColorDepth, frame::Frame, prelude::*};
use std::{cell::RefCell, rc::Rc};

use crate::{e4command::E4Command, e4config::E4Config, e4icon::E4Icon, error};

/// The configuration for a [E4Button].
pub struct E4ButtonConfig {
    /// The [E4Command] containing the command and the args to exec.
    pub command: E4Command,
    /// The path of the [E4Icon] image for the [E4Button].
    pub icon_path: String,
}

/// A fltk [Button] improved with a [E4Command].
pub struct E4Button {
    /// The fltk [Button]
    pub button: Button,
    /// The [E4Command] enclosed in a [Rc] to allow shared ownership and a [RefCell] to allow interior mutability through the [E4Button::set_command] implementation.
    pub command: Rc<RefCell<E4Command>>,
}

impl E4Button {
    /// Create a new [E4Button].
    ///
    /// # Example
    ///
    /// Create a [E4Button] of size 64x64 at position 0, 0.
    ///
    /// ```rust
    /// use e4docker::{e4button::E4Button, e4command::E4Command};
    /// use e4docker::{e4config::E4Config, e4icon::E4Icon};
    /// use fltk::{frame::Frame, prelude::*};
    /// use std::{cell::RefCell, rc::Rc, path::PathBuf};
    ///
    /// // Read the global configuration
    /// let directory = PathBuf::from("~")
    ///                     .join(".config")
    ///                     .join("e4docker")
    ///                     .join("config");
    /// let config = E4Config::read(&project_config_dir);
    /// let frame = Frame::default();
    /// let command = E4Command::new(String::from("/usr/bin/nano"), vec![]);
    /// let command = Rc::new(RefCell::new(command));
    /// let icon = E4Icon::new(PathBuf::from("icon.png"), 64, 64);
    ///
    /// let my_button = E4Button::new(
    ///     x: 0,
    ///     y: 0,
    ///     parent: &frame,
    ///     command,
    ///     config: &config,
    ///     icon,
    /// );
    /// ```
    pub fn new(
        x: i32,
        y: i32,
        parent: &Frame,
        command: Rc<RefCell<E4Command>>,
        config: &E4Config,
        icon: E4Icon,
    ) -> Self {
        let mut button = Button::default()
            .with_pos(x, y)
            .with_size(icon.width(), icon.height())
            .center_y(parent);
        let (x, y) = (button.x(), button.y());
        let command_clone = Rc::clone(&command);
        button.set_callback(move |_| {
            let result = command_clone
                .borrow()
                .exec();
            match result {
                Ok(_) => (),
                Err(e) => {
                    let message = format!("Failed to execute command  {}: {}", command_clone.borrow().get_cmd(), e.to_string());
                    error(&message);
                },
            };
        });
        let button_icon = image::open(config.assets_dir.join(icon.path()))
            .unwrap_or_else(|_| panic!("Cannot find {:?}", config.assets_dir.join(icon.path())));
        button.draw(move |_| {
            draw::draw_image(
                &button_icon.to_rgba8(),
                x,
                y,
                icon.width(),
                icon.height(),
                ColorDepth::Rgba8,
            )
            .unwrap();
        });
        E4Button { button, command }
    }

    /// Read the configuration of a [E4Button] from confi/button_name.conf.
    /// Return an instance of a [E4ButtonConfig], containing the fltk [Button] and the [E4Command].
    pub fn read_config(config: &E4Config, button_name: &String) -> E4ButtonConfig {
        // Read config.config_dir/button_name.conf
        let mut config_file = config.config_dir.join(button_name);
        config_file.set_extension("conf");
        let mut config = Ini::new();
        let result = config
            .load(config_file);

        match result {
            Ok(_) => (),
            Err(e) => {
                let message = format!("Cannot load the button config file: {}", e.to_string());
                error(&message);
            },
        };

        // Get the fields
        let command: String = config.get("BUTTON", "COMMAND").unwrap();
        let icon_path: String = config.get("BUTTON", "ICON").unwrap();
        let number_of_args: i32 = config
            .get("BUTTON", "NUMBER_OF_ARGS")
            .unwrap()
            .parse()
            .unwrap();
        // Read the args
        let mut args = vec![];
        for n in 1..=number_of_args {
            let arg_key = format!("arg{}", n);
            let argument = config.get("BUTTON", &arg_key).unwrap();
            args.push(argument);
        }
        // Create the E4Command
        let command = E4Command::new(command, args);
        E4ButtonConfig { command, icon_path }
    }

    /// Set a new command for the [E4Button]
    pub fn set_command(&self, cmd: String, args: Vec<String>) {
        self.command.borrow_mut().set(cmd, args);
    }
}
