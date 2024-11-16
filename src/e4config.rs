use configparser::ini::Ini;
use std::path::{Path,PathBuf};
use crate::error;

/// The configuration of e4docker read from e4docker.conf
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
}

impl E4Config {
    /// Read the configuration from config_dir/e4docker.conf
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
                let message = format!("Cannot load e4docker.conf: {}", e.to_string());
                error(&message);
            },
        };

        // Read the number of buttons
        let number_of_buttons: i32 = config
            .get("E4DOCKER", "NUMBER_OF_BUTTONS")
            .unwrap()
            .parse()
            .unwrap();
        // Read the margin between the buttons
        let margin_between_buttons: i32 = config
            .get("E4DOCKER", "MARGIN_BETWEEN_BUTTONS")
            .unwrap()
            .parse()
            .unwrap();
        // Read the margin between the frame and the window
        let frame_margin: i32 = config
            .get("E4DOCKER", "FRAME_MARGIN")
            .unwrap()
            .parse()
            .unwrap();
        // Read the buttons
        let mut buttons = vec![];
        for n in 1..=number_of_buttons {
            let button_key = format!("button{}", n);
            let button_name = config.get("BUTTONS", &button_key).unwrap();
            buttons.push(button_name);
        }
        // Read the buttons width (the same as the icons width)
        let icon_width: i32 = config
            .get("E4DOCKER", "ICON_WIDTH")
            .unwrap()
            .parse()
            .unwrap();
        // Read the buttons height (the same as the icons height)
        let icon_height: i32 = config
            .get("E4DOCKER", "ICON_HEIGHT")
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
            assets_dir: config_dir.join("assets"),
            margin_between_buttons,
            frame_margin,
            window_width,
            window_height,
            icon_width,
            icon_height,
        }
    }
}
