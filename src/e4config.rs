use configparser::ini::Ini;
use std::{env, path::{Path,PathBuf}, process::Command};
use fltk::{app, prelude::*, window::Window};
use crate::e4initialize;

/// Sections in the configuration files.
/// e4docker.conf.
pub const E4DOCKER_DOCKER_SECTION: &str = "E4DOCKER";
pub const E4DOCKER_BUTTON_SECTION: &str = "BUTTONS";

/// A button configuration file.
pub const BUTTON_BUTTON_SECTION: &str = "BUTTON";

/// The configuration of e4docker read from e4docker.conf.
pub struct E4Config {
    pub config_dir: PathBuf,
    pub buttons: Vec<String>,
    pub assets_dir: PathBuf,
    pub margin_between_buttons: i32,
    pub frame_margin: i32,
    pub window_width: i32,
    pub window_height: i32,
    pub icon_width: i32,
    pub icon_height: i32,
    pub x: i32,
    pub y: i32,
}

/// Create the about dialog.
pub fn create_about_dialog(message: &str) {
    let mut wind = Window::default()
        .with_size(500, 300)
        .with_label("About");

    // Create TextDisplay for the message
    let mut text_display = fltk::text::TextDisplay::new(10, 10, 480, 230, "");
    let mut buff = fltk::text::TextBuffer::default();
    buff.set_text(message);
    text_display.set_buffer(buff);
    text_display.set_scrollbar_size(15);
    text_display.wrap_mode(fltk::text::WrapMode::AtBounds, 0); // Corretto: usando WrapMode::A

    // Add OK button at the bottom
    let mut ok_btn = fltk::button::Button::new(200, 250, 100, 30, "OK");
    ok_btn.set_callback({
        let mut wind = wind.clone();
        move |_| wind.hide()
    });

    wind.make_modal(true);
    wind.end();
    wind.show();

    // Run modal window
    while wind.shown() {
        app::wait();
    }
}

/// Restart the program.
pub fn restart_app() {
    // Get the current exe
    let current_exe = env::current_exe().expect("Failed to get current executable path");

    // Get the args
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        // Start a child process
        let _ = Command::new(&current_exe)
            .args(&args[1..])
            .spawn()
            .expect("Failed to restart the program");
    } else {
        // Start a child process
        let _ = Command::new(&current_exe)
            .spawn()
            .expect("Failed to restart the program");
    }
    // End the current process
    std::process::exit(0);
}

/// Get a temporary file name for storing temporary configuration data.
pub fn get_tmp_file() -> PathBuf {
    let package_name = env!("CARGO_PKG_NAME");
    let mut tmp_file = std::env::temp_dir().join(package_name);
    tmp_file.set_extension("conf");
    tmp_file
}

impl std::clone::Clone for E4Config {
    fn clone(&self) -> Self {
        Self {
            config_dir: self.config_dir.clone(),
            buttons: self.buttons.clone(),
            assets_dir: self.assets_dir.clone(),
            margin_between_buttons: self.margin_between_buttons,
            frame_margin: self.frame_margin,
            window_width: self.window_width,
            window_height: self.window_height,
            icon_width: self.icon_width,
            icon_height: self.icon_height,
            x: self.x,
            y: self.y,
        }
    }
}

impl E4Config {
    /// Read the configuration from config_dir/e4docker.conf.
    pub fn read(config_dir: &Path) -> Self {
        // Read the config file
        let package_name = env!("CARGO_PKG_NAME");
        let mut config_file = config_dir.join(package_name);
        config_file.set_extension("conf");
        let mut config = Ini::new();
        let result = config
            .load(config_file);
        match result {
            Ok(_) => (),
            Err(e) => {
                let message = format!("Cannot load e4docker.conf: {}", e);
                fltk::dialog::alert_default(&message);
            },
        };

        // Read the x position of the window
        let mut x: i32 = 0;
        let mut y: i32 = 0;
        if let Some(val) = config.get(E4DOCKER_DOCKER_SECTION, "X") {
            x = val.parse().unwrap();
        }
        if let Some(val) = config.get(E4DOCKER_DOCKER_SECTION, "Y") {
            y = val.parse().unwrap();
        }
        // Read the number of buttons
        let number_of_buttons: i32 = config
            .get(E4DOCKER_DOCKER_SECTION, "NUMBER_OF_BUTTONS")
            .unwrap()
            .parse()
            .unwrap();
        // Read the margin between the buttons
        let margin_between_buttons: i32 = config
            .get(E4DOCKER_DOCKER_SECTION, "MARGIN_BETWEEN_BUTTONS")
            .unwrap()
            .parse()
            .unwrap();
        // Read the margin between the frame and the window
        let frame_margin: i32 = config
            .get(E4DOCKER_DOCKER_SECTION, "FRAME_MARGIN")
            .unwrap()
            .parse()
            .unwrap();
        // Read the buttons
        let mut buttons = vec![];
        for n in 1..=number_of_buttons {
            let button_key = format!("button{}", n);
            let button_name = config.get(E4DOCKER_BUTTON_SECTION, &button_key).unwrap();
            buttons.push(button_name);
        }
        // Read the buttons width (the same as the icons width)
        let icon_width: i32 = config
            .get(E4DOCKER_DOCKER_SECTION, "ICON_WIDTH")
            .unwrap()
            .parse()
            .unwrap();
        // Read the buttons height (the same as the icons height)
        let icon_height: i32 = config
            .get(E4DOCKER_DOCKER_SECTION, "ICON_HEIGHT")
            .unwrap()
            .parse()
            .unwrap();
        // Calculates the window width
        let window_width = (number_of_buttons * icon_width)
            + (number_of_buttons * margin_between_buttons)
            + (frame_margin * 2);
        // Calculates the window height, adding margin * 4 for the 4 sides frame margin
        let window_height = icon_height + (frame_margin * 4);
        // Return the configuration
        Self {
            config_dir: config_dir.to_path_buf(),
            buttons,
            assets_dir: e4initialize::get_package_assets_dir(),
            margin_between_buttons,
            frame_margin,
            window_width,
            window_height,
            icon_width,
            icon_height,
            x,
            y,
        }
    }

    /// Get a value from the configuration file.
    pub fn get_value(&mut self, section: String, key: String) -> String {
        // Read the config file
        let package_name = env!("CARGO_PKG_NAME");
        let mut config_file = self.config_dir.join(package_name);
        config_file.set_extension("conf");
        let mut config = Ini::new();
        let result = config
            .load(&config_file);
        match result {
            Ok(_) => (),
            Err(e) => {
                let message = format!("Cannot load e4docker.conf: {}", e);
                fltk::dialog::alert_default(&message);
            },
        };
        // Get and return the key and the value
        config.get(&section, &key).unwrap()
    }

    /// Save the buttons in config_dir/e4docker.conf.
    pub fn save_buttons(&mut self, buttons: &[String]) {
        // Save the buttons
        for (i, button) in buttons.iter().enumerate() {
            let key = format!("button{}", i+1);
            self.set_value(E4DOCKER_BUTTON_SECTION.to_string(), key, Some(button.to_string()));
        }
    }

    /// Set a value in the configuration file.
    pub fn set_value(&mut self, section: String, key: String, value: Option<String>) {
        // Read the config file
        let package_name = env!("CARGO_PKG_NAME");
        let mut config_file = self.config_dir.join(package_name);
        config_file.set_extension("conf");
        let mut config = Ini::new();
        let result = config
            .load(&config_file);
        match result {
            Ok(_) => (),
            Err(e) => {
                let message = format!("Cannot load e4docker.conf: {}", e);
                fltk::dialog::alert_default(&message);
            },
        };
        // Set the key and the value
        config.set(&section, &key, value);
        config.write(config_file).expect("Cannot save e4docker.conf");
    }

    /// Get the number of buttons in the configuration file
    pub fn get_number_of_buttons(&mut self) -> i32 {
        let number_of_buttons: i32 = self.get_value(E4DOCKER_DOCKER_SECTION.to_string(), String::from("NUMBER_OF_BUTTONS")).parse().unwrap();
        number_of_buttons
    }

    /// Set the number of buttons in the configuration file
    pub fn set_number_of_buttons(&mut self, number: i32) {
        self.set_value(E4DOCKER_DOCKER_SECTION.to_string(), String::from("NUMBER_OF_BUTTONS"), Some(number.to_string()));
    }

    /// Delete a key from the configuratio file.
    pub fn remove_key(&mut self, section: String, key: String) {
        let package_name = env!("CARGO_PKG_NAME");
        let mut config_file = self.config_dir.join(package_name);
        config_file.set_extension("conf");
        let mut config = Ini::new();
        let result = config
            .load(&config_file);
        match result {
            Ok(_) => (),
            Err(e) => {
                let message = format!("Cannot load e4docker.conf: {}", e);
                fltk::dialog::alert_default(&message);
            },
        };
        config.remove_key(&section, &key);
        config.write(config_file).expect("Cannot save e4docker.conf");
    }
}
