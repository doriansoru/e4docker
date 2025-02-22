use crate::{e4initialize, tr, translations::Translations};
use configparser::ini::Ini;
use fltk::{app, misc::Spinner, prelude::*, window::Window};
use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
    sync::{Arc, Mutex},
    thread,
};

/// Sections in the configuration files.
/// e4docker.conf.
pub const E4DOCKER_DOCKER_SECTION: &str = "E4DOCKER";
pub const E4DOCKER_BUTTON_SECTION: &str = "BUTTONS";

const E4DOCKER_MARGIN_BETWEEN_BUTTONS: &str = "MARGIN_BETWEEN_BUTTONS";
const E4DOCKER_FRAME_MARGIN: &str = "FRAME_MARGIN";
const E4DOCKER_ICON_WIDTH: &str = "ICON_WIDTH";
const E4DOCKER_ICON_HEIGHT: &str = "ICON_HEIGHT";

/// A button configuration file.
pub const BUTTON_BUTTON_SECTION: &str = "BUTTON";

// Definisci un tipo di errore personalizzato
#[derive(Debug)]
struct E4Error {
    details: String,
}

// Implementa il tratto `std::fmt::Display` per il tuo tipo di errore
impl std::fmt::Display for E4Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl std::error::Error for E4Error {}

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
pub fn create_about_dialog(message: &str, translations: Arc<Mutex<Translations>>) {
    let mut wind = Window::default().with_size(500, 300).with_label(&tr!(
        translations,
        get_or_default,
        "about",
        "About"
    ));

    // Create TextDisplay for the message
    let mut text_display = fltk::text::TextDisplay::new(10, 10, 480, 230, "");
    let mut buff = fltk::text::TextBuffer::default();
    buff.set_text(message);
    text_display.set_buffer(buff);
    text_display.set_scrollbar_size(15);
    text_display.wrap_mode(fltk::text::WrapMode::AtBounds, 0); // Corretto: usando WrapMode::A

    // Add OK button at the bottom
    let mut ok_btn = fltk::button::Button::new(
        200,
        250,
        100,
        30,
        tr!(translations, get_or_default, "ok", "OK").as_str(),
    );
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
pub fn restart_app(translations: Arc<Mutex<Translations>>) {
    // Get the current exe
    let current_exe = env::current_exe().expect(&tr!(
        translations,
        get_or_default,
        "failed-to-get-current-executable-path",
        "Failed to get current executable path"
    ));

    // Get the args
    let args: Vec<String> = env::args().collect();

    let translations_clone = translations.clone();

    if args.len() > 1 {
        thread::spawn(move || {
            let _ = Command::new(&current_exe)
                .args(&args[1..])
                .spawn()
                .expect(&tr!(
                    translations_clone,
                    get_or_default,
                    "failed-to-restart-the-program",
                    "Failed to restart the program"
            ));
            // End the current process
            std::process::exit(0);
        });
    } else {
        thread::spawn(move || {
            let _ = Command::new(&current_exe).spawn().expect(&tr!(
                translations_clone,
                get_or_default,
                "failed-to-restart-the-program",
                "Failed to restart the program"
            ));

            // End the current process
            std::process::exit(0);
        });
    }
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
    /// Creates and manages the settings dialog
    pub fn create_settings_dialog(
        &mut self,
        translations: Arc<Mutex<Translations>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut window = Window::default().with_size(700, 300);
        let mut grid = fltk_grid::Grid::default()
            .with_size(650, 250)
            .center_of(&window);
        grid.show_grid(false);
        grid.set_gap(10, 10);
        let grid_values = [self.icon_width as f64, self.icon_height as f64];
        let ncols = 2;
        let nrows = 3;
        grid.set_layout(nrows, ncols);

        let labels = [
            &tr!(translations, get_or_default, "icon-width", "Icon width"),
            &tr!(translations, get_or_default, "icon-height", "Icon height"),
        ];

        // Populates the grid
        let mut icon_width_label = fltk::frame::Frame::default().with_label(labels[0]);
        let mut icon_width_input = Spinner::default();
        icon_width_input.set_step(1.0);
        icon_width_input.set_range(16.0, 512.0);
        icon_width_input.set_value(grid_values[0]);
        grid.set_widget(&mut icon_width_label, 0, 0)?;
        grid.set_widget(&mut icon_width_input, 0, 1)?;

        let mut icon_height_label = fltk::frame::Frame::default().with_label(labels[1]);
        let mut icon_height_input = Spinner::default();
        icon_height_input.set_step(1.0);
        icon_height_input.set_range(16.0, 512.0);
        icon_height_input.set_value(grid_values[1]);
        grid.set_widget(&mut icon_height_label, 1, 0)?;
        grid.set_widget(&mut icon_height_input, 1, 1)?;

        // Add Save button at the bottom
        let mut save_button = fltk::button::Button::new(
            200,
            250,
            100,
            30,
            tr!(translations, get_or_default, "save", "Save").as_str(),
        );
        grid.set_widget(&mut save_button, 2, 0..2)?;

        save_button.set_callback({
            let mut wind = window.clone();
            let mut myself = self.clone();
            move |_| {
                let icon_width = (icon_width_input.value() as i32).to_string();
                let icon_height = (icon_height_input.value() as i32).to_string();
                wind.hide();
                myself.set_value(
                    E4DOCKER_DOCKER_SECTION.to_string(),
                    E4DOCKER_ICON_WIDTH.to_string(),
                    Some(icon_width),
                    translations.clone(),
                );
                myself.set_value(
                    E4DOCKER_DOCKER_SECTION.to_string(),
                    E4DOCKER_ICON_HEIGHT.to_string(),
                    Some(icon_height),
                    translations.clone(),
                );
                crate::e4config::restart_app(translations.clone());
            }
        });

        window.make_modal(true);
        window.end();
        window.show();
        // Run modal window
        while window.shown() {
            app::wait();
        }
        Ok(())
    }

    /// Read the configuration from config_dir/e4docker.conf.
    pub fn read(
        config_dir: &Path,
        translations: Arc<Mutex<Translations>>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Read the config file
        let package_name = env!("CARGO_PKG_NAME");
        let mut config_file = config_dir.join(package_name);
        config_file.set_extension("conf");
        let mut config = Ini::new();
        let _ = config.load(config_file)?;

        // Read the x position of the window
        let mut x: i32 = 0;
        let mut y: i32 = 0;
        let mut number_of_buttons: i32 = 0;
        let mut margin_between_buttons: i32 = 0;
        let mut frame_margin: i32 = 0;
        let mut icon_width: i32 = 0;
        let mut icon_height: i32 = 0;

        // Read the x coordinate of the docker
        if let Some(val) = config.get(E4DOCKER_DOCKER_SECTION, "X") {
            x = val.parse()?;
        }

        // Read the y coordinate of the docker
        if let Some(val) = config.get(E4DOCKER_DOCKER_SECTION, "Y") {
            y = val.parse()?;
        }

        // Read the number of buttons
        if let Some(val) = config.get(E4DOCKER_DOCKER_SECTION, "NUMBER_OF_BUTTONS") {
            number_of_buttons = val.parse()?;
        };

        // Read the margin between the buttons
        if let Some(val) = config.get(E4DOCKER_DOCKER_SECTION, E4DOCKER_MARGIN_BETWEEN_BUTTONS) {
            margin_between_buttons = val.parse()?;
        };

        // Read the margin between the buttons
        if let Some(val) = config.get(E4DOCKER_DOCKER_SECTION, E4DOCKER_FRAME_MARGIN) {
            frame_margin = val.parse()?;
        };

        // Read the buttons
        let mut buttons = vec![];
        for n in 1..=number_of_buttons {
            let button_key = format!("button{}", n);
            let mut button_name: String = "".to_string();
            if let Some(val) = config.get(E4DOCKER_BUTTON_SECTION, &button_key) {
                button_name = val;
            };
            buttons.push(button_name);
        }

        // Read the buttons width (the same as the icons width)
        if let Some(val) = config.get(E4DOCKER_DOCKER_SECTION, E4DOCKER_ICON_WIDTH) {
            icon_width = val.parse()?;
        };

        // Read the buttons height (the same as the icons height)
        if let Some(val) = config.get(E4DOCKER_DOCKER_SECTION, E4DOCKER_ICON_HEIGHT) {
            icon_height = val.parse()?;
        };

        // Calculates the window width
        let window_width = (number_of_buttons * icon_width)
            + (number_of_buttons * margin_between_buttons)
            + (frame_margin * 2);

        // Calculates the window height, adding margin * 4 for the 4 sides frame margin
        let window_height = icon_height + (frame_margin * 4);

        // Return the configuration
        Ok(Self {
            config_dir: config_dir.to_path_buf(),
            buttons,
            assets_dir: e4initialize::get_package_assets_dir(translations.clone()),
            margin_between_buttons,
            frame_margin,
            window_width,
            window_height,
            icon_width,
            icon_height,
            x,
            y,
        })
    }

    /// Get a value from the configuration file.
    pub fn get_value(
        &mut self,
        section: String,
        key: String,
        translations: Arc<Mutex<Translations>>,
    ) -> Option<String> {
        // Read the config file
        let package_name = env!("CARGO_PKG_NAME");
        let mut config_file = self.config_dir.join(package_name);
        config_file.set_extension("conf");
        let mut config = Ini::new();
        let result = config.load(&config_file);
        match result {
            Ok(_) => (),
            Err(e) => {
                let message = tr!(
                    translations,
                    format,
                    "cannot-load-e4docker-conf",
                    &[&e.to_string()]
                );
                fltk::dialog::alert_default(&message);
            }
        };
        // Get and return the key and the value
        config.get(&section, &key)
    }

    /// Save the buttons in config_dir/e4docker.conf.
    pub fn save_buttons(&mut self, buttons: &[String], translations: Arc<Mutex<Translations>>) {
        // Save the buttons
        for (i, button) in buttons.iter().enumerate() {
            let key = format!("button{}", i + 1);
            self.set_value(
                E4DOCKER_BUTTON_SECTION.to_string(),
                key,
                Some(button.to_string()),
                translations.clone(),
            );
        }
    }

    pub fn swap_buttons(
        &mut self,
        buttons: &mut [String],
        first_button_index: usize,
        second_button_index: usize,
        translations: Arc<Mutex<Translations>>,
    ) {
        let temp_button = buttons[first_button_index].clone();
        buttons[first_button_index] = buttons[second_button_index].clone();
        buttons[second_button_index] = temp_button;
        self.save_buttons(buttons, translations.clone());
        crate::e4config::restart_app(translations.clone())
    }

    /// Set a value in the configuration file.
    pub fn set_value(
        &mut self,
        section: String,
        key: String,
        value: Option<String>,
        translations: Arc<Mutex<Translations>>,
    ) {
        // Read the config file
        let package_name = env!("CARGO_PKG_NAME");
        let mut config_file = self.config_dir.join(package_name);
        config_file.set_extension("conf");
        let mut config = Ini::new();
        let result = config.load(&config_file);
        match result {
            Ok(_) => (),
            Err(e) => {
                let message = tr!(
                    translations,
                    format,
                    "cannot-load-e4docker-conf",
                    &[&e.to_string()]
                );
                fltk::dialog::alert_default(&message);
            }
        };
        // Set the key and the value
        config.set(&section, &key, value);
        config.write(config_file).expect(&tr!(
            translations,
            get_or_default,
            "cannot-save-e4docker-conf",
            "Cannot save e4docker.conf"
        ));
    }

    /// Get the number of buttons in the configuration file
    pub fn get_number_of_buttons(
        &mut self,
        translations: Arc<Mutex<Translations>>,
    ) -> Result<i32, Box<dyn std::error::Error>> {
        let number_of_buttons: i32;
        if let Some(val) = self.get_value(
            E4DOCKER_DOCKER_SECTION.to_string(),
            String::from("NUMBER_OF_BUTTONS"),
            translations.clone(),
        ) {
            number_of_buttons = val.parse()?;
        } else {
            return Err(Box::new(E4Error {
                details: tr!(
                    translations,
                    get_or_default,
                    "cannot-get-the-number-of-buttons",
                    "Cannot get the number of buttons"
                ),
            }));
        };
        Ok(number_of_buttons)
    }

    /// Set the number of buttons in the configuration file
    pub fn set_number_of_buttons(&mut self, number: i32, translations: Arc<Mutex<Translations>>) {
        self.set_value(
            E4DOCKER_DOCKER_SECTION.to_string(),
            String::from("NUMBER_OF_BUTTONS"),
            Some(number.to_string()),
            translations.clone(),
        );
    }

    /// Delete a key from the configuratio file.
    pub fn remove_key(
        &mut self,
        section: String,
        key: String,
        translations: Arc<Mutex<Translations>>,
    ) {
        let package_name = env!("CARGO_PKG_NAME");
        let mut config_file = self.config_dir.join(package_name);
        config_file.set_extension("conf");
        let mut config = Ini::new();
        let result = config.load(&config_file);
        match result {
            Ok(_) => (),
            Err(e) => {
                let message = tr!(
                    translations,
                    format,
                    "cannot-load-e4docker-conf",
                    &[&e.to_string()]
                );
                fltk::dialog::alert_default(&message);
            }
        };
        config.remove_key(&section, &key);
        config.write(config_file).expect(&tr!(
            translations,
            get_or_default,
            "cannot-save-e4docker-conf",
            "Cannot save e4docker.conf"
        ));
    }
}
