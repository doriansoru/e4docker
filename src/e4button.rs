use configparser::ini::Ini;
use fltk::{app, button::Button, draw, enums::ColorDepth, frame::Frame, prelude::*, window::Window};
use std::{cell::RefCell, path::PathBuf, rc::Rc};
use crate::{e4command::E4Command, e4config::E4Config, e4icon::E4Icon};
use round::round;

/// The configuration for a [E4Button].
pub struct E4ButtonConfig {
    /// The [E4Command] containing the command and the args to exec.
    pub command: E4Command,
    /// The path of the [E4Icon] image for the [E4Button].
    pub icon_path: String,
}

/// A fltk [Button] improved with a [E4Command].
pub struct E4Button {
    /// The name of the button, corresponding to the .conf file name
    pub name: String,
    /// The x position of the button
    pub x: i32,
    /// The y position of the button
    pub y: i32,
    /// The width of the button
    pub width: i32,
    /// The height of the button
    pub height: i32,
    /// The fltk [Button]
    pub button: Button,
    /// The button icon
    pub icon: E4Icon,
    /// The [E4Command] enclosed in a [Rc] to allow shared ownership and a [RefCell] to allow interior mutability through the [E4Button::set_command] implementation.
    pub command: Rc<RefCell<E4Command>>,
}


/// Create the [E4Button]s.
pub fn create_buttons(config: &E4Config, wind: &mut Window, frame: &Frame) -> Vec<E4Button> {
    let mut buttons = vec![];
    let mut current_e4button;
    // Put the buttons in the window
    let mut x = config.margin_between_buttons;
    let y: i32 = round(
        (config.window_height as f64 - config.icon_height as f64) / 2.0,
        0,
    ) as i32;
    for button_name in &config.buttons {
        // Read the button config
        let button_config: E4ButtonConfig = E4Button::read_config(config, &button_name);
        // Create the icon
        let icon = E4Icon::new(
            PathBuf::from(button_config.icon_path),
            config.icon_width,
            config.icon_height,
        );
        // Create the command
        let command = Rc::new(RefCell::new(button_config.command));
        // Create the button
        current_e4button = E4Button::new(
            button_name,
            x,
            y,
            &frame,
            Rc::clone(&command),
            &config,
            icon,
        );
        // Add the button to the window
        wind.add(&current_e4button.button);
        buttons.push(current_e4button);
        x += config.icon_width + config.margin_between_buttons;
    }
    buttons
}

/// Clone trait for [E4Button].
impl std::clone::Clone for E4Button {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            x: self.x,
            y: self.y,
            width: self.width,
            height: self.height,
            button: self.button.clone(),
            icon: self.icon.clone(),
            command: self.command.clone(),
        }
    }
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
    ///     name: "nano".to_string(),
    ///     x: 0,
    ///     y: 0,
    ///     parent: &frame,
    ///     command,
    ///     config: &config,
    ///     icon,
    /// );
    /// ```
    pub fn new(
        name: &String,
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
                .borrow_mut()
                .exec();
            match result {
                Ok(_) => (),
                Err(e) => {
                    let message = format!("Failed to execute command  {}: {}", command_clone.borrow().get_cmd(), e.to_string());
                    fltk::dialog::alert_default(&message);
                },
            };
        });

        let button_icon;
        // If the icon path does not exist, search for the icon in the assets directory
        if !icon.path().exists() {
            button_icon = image::open(config.assets_dir.join(icon.path()))
                .unwrap_or_else(|_| panic!("Cannot find {:?}", config.assets_dir.join(icon.path())));
        } else {
            button_icon = image::open(icon.path())
                .unwrap_or_else(|_| panic!("Cannot find {:?}", config.assets_dir.join(icon.path())));
        }
        let (w, h) = (icon.width(), icon.height());

        button.draw(move |_| {
            draw::draw_image(
                &button_icon.to_rgba8(),
                x,
                y,
                w,
                h,
                ColorDepth::Rgba8,
            )
            .unwrap();
        });
        E4Button {
            name: name.to_string(),
            x,
            y,
            width: w,
            height: h,
            button,
            icon,
            command
        }
    }

    /// Set a new command for the [E4Button].
    pub fn set_command(&self, cmd: String, arguments: String) {
        self.command.borrow_mut().set(cmd, arguments);
    }

    /// Delete the [E4Button].
    pub fn delete(&mut self, config: &mut E4Config) {
        // Delete the button configuration file
        let mut config_file = PathBuf::from(&self.name).with_extension("");
        config_file.set_extension("conf");
        config_file = config.config_dir.join(config_file);
        std::fs::remove_file(&config_file).unwrap();

        // Delete the icon
        self.icon.delete(&config);

        // Create a new buttons vec removing the one to be deleted
        let mut buttons = vec![];
        let old_buttons = config.buttons.clone();
        for (i, button) in old_buttons.iter().enumerate() {
            let button_number = i + 1;
            if *button != self.name {
                buttons.push(button);
            }
            let key_to_remove = format!("button{}", button_number);
            config.remove_key(crate::e4config::E4DOCKER_BUTTON_SECTION.to_string(), key_to_remove);
        }
        config.set_number_of_buttons(buttons.len() as i32);
        for (i, button) in buttons.iter().enumerate() {
            let button_number = i + 1;
            let button_name = button.to_string();
            config.set_value(crate::e4config::E4DOCKER_BUTTON_SECTION.to_string(), format!("button{}", button_number), Some(button_name));
        }
        crate::e4config::restart_app();
    }

    /// Edit the [E4Button].
    pub fn edit(&mut self, config: &mut E4Config) {
        let mut config_file = config.config_dir.join(&self.name);
        config_file.set_extension("conf");
        let tmp_file_path = crate::e4config::get_tmp_file();
        std::fs::copy(config_file.clone(), tmp_file_path).unwrap();
        let mut wind = Window::default()
            .with_size(500, 300)
            .with_label(format!("Edit {}", self.name).as_str());
        let mut grid = fltk_grid::Grid::default().with_size(450, 250).center_of(&wind);
        grid.show_grid(false);
        grid.set_gap(10, 10);
        let command = self.command.borrow();
        let icon = self.icon.path().display().to_string();
        let grid_values = vec![&self.name, &icon, command.get_cmd(), command.get_arguments()];
        let ncells = grid_values.len() * 2 + 1; // Label and text for each element + ok button
        let ncols = 2;
        let nrows = (ncells as f64 / ncols as f64).ceil() as i32;
        grid.set_layout(nrows, ncols);

        let labels = vec!["Name", "Icon", "Command", "Arguments"];

        // Populates the grid
        let mut name_label = fltk::frame::Frame::default().with_label(labels[0]);
        let mut name_input = fltk::input::Input::default();
        name_input.set_value(grid_values[0]);
        grid.set_widget(&mut name_label, 0, 0).unwrap();
        grid.set_widget(&mut name_input, 0, 1).unwrap();

        let mut icon_label = fltk::frame::Frame::default().with_label(labels[1]);
        let icon_path = &config.assets_dir.join(self.icon.path());
        let mut image = fltk::image::PngImage::load(&icon_path).unwrap();
        image.scale(self.width, self.height, true, true);
        let mut button_icon = fltk::button::Button::default().with_size(self.width, self.height);
        button_icon.set_image(Some(image));

        // Usa un Rc per condividere lo stato tra il callback e il resto del codice
        let icon_path = Rc::new(RefCell::new(icon_path.clone()));
        let icon_path_clone = Rc::clone(&icon_path);

        let assets_dir = config.assets_dir.clone();
        let (w, h) = (self.width, self.height);
        button_icon.set_callback(move |b| {
            let mut chooser = fltk::dialog::FileChooser::new(
                &assets_dir,                    // directory
                "*.png",                    // filter or pattern
                fltk::dialog::FileChooserType::Single, // chooser type
                "Choose icon",     // title
            );
            chooser.show();
            while chooser.shown() {
                app::wait();
            }
            if !chooser.value(1).is_none() {
                let image_path = chooser.value(1).unwrap();
                let mut new_image = fltk::image::PngImage::load(&image_path).unwrap();
                new_image.scale(w, h, true, true);
                b.set_image(Some(new_image));
                *icon_path_clone.borrow_mut() = std::path::PathBuf::from(&image_path);
                b.redraw();
                let mut config = Ini::new();
                let tmp_file_path = crate::e4config::get_tmp_file();
                let result = config
                    .load(&tmp_file_path);
                config.set(crate::e4config::BUTTON_BUTTON_SECTION, "icon", Some(image_path));
                config.write(&tmp_file_path).expect("Cannot save the config file.");

                match result {
                    Ok(_) => (),
                    Err(e) => {
                        let message = format!("Cannot load the button config file: {}", e.to_string());
                        fltk::dialog::alert_default(&message);
                    },
                };
            }
        });

        grid.set_widget(&mut icon_label, 1, 0).unwrap();
        grid.set_widget(&mut button_icon, 1, 1).unwrap();

        let mut command_label = fltk::frame::Frame::default().with_label(labels[2]);
        let mut command_input = fltk::input::Input::default();
        command_input.set_value(grid_values[2]);
        grid.set_widget(&mut command_label, 2, 0).unwrap();
        grid.set_widget(&mut command_input, 2, 1).unwrap();

        let mut arguments_label = fltk::frame::Frame::default().with_label(labels[3]);
        let mut arguments_input = fltk::input::Input::default();
        arguments_input.set_value(command.get_arguments());
        grid.set_widget(&mut arguments_label, 3, 0).unwrap();
        grid.set_widget(&mut arguments_input, 3, 1).unwrap();

        // Add OK button at the bottom
        let mut save_button = fltk::button::Button::new(200, 250, 100, 30, "Save");
        let config_clone = config.clone();
        save_button.set_callback({
            let mut wind = wind.clone();
            move |_| {
                wind.hide();
                let tmp_file_path = crate::e4config::get_tmp_file();
                let mut tmp_config = Ini::new();
                let _ = tmp_config
                    .load(&tmp_file_path);
                let name = name_input.value();
                let mut config_file = config_clone.config_dir.join(name);
                config_file.set_extension("conf");
                let command = command_input.value();
                let arguments = arguments_input.value();
                tmp_config.set(crate::e4config::BUTTON_BUTTON_SECTION, "command", Some(command));
                tmp_config.set(crate::e4config::BUTTON_BUTTON_SECTION, "arguments", Some(arguments));
                tmp_config.write(&tmp_file_path).expect(format!("Cannot save {}", &tmp_file_path.display()).as_str());
                std::fs::copy(tmp_file_path, &config_file).unwrap();
                crate::e4config::restart_app();
            }
        });
        grid.set_widget(&mut save_button, 4, 0..2).unwrap();

        wind.make_modal(true);
        wind.end();
        wind.show();

        // Run modal window
        while wind.shown() {
            app::wait();
        }
    }

    /// Create a new [E4Button] after sibling.
    pub fn new_button(config: &mut E4Config, sibling: &E4Button) {
        let name = "generic";
        let mut config_file = config.config_dir.join(name);
        config_file.set_extension("conf");
        let tmp_file_path = crate::e4config::get_tmp_file();
        std::fs::copy(config_file.clone(), tmp_file_path).unwrap();
        let button_config = Self::read_config(&config, &name.to_string());
        let mut wind = Window::default()
            .with_size(500, 300)
            .with_label(format!("Edit {}", name).as_str());
        let mut grid = fltk_grid::Grid::default().with_size(450, 250).center_of(&wind);
        grid.show_grid(false);
        grid.set_gap(10, 10);
        let command = button_config.command;
        let icon = button_config.icon_path;
        let grid_values = vec![name, &icon, command.get_cmd(), command.get_arguments()];
        let ncells = 9; // Label and text for each element + ok button
        let ncols = 2;
        let nrows = (ncells as f64 / ncols as f64).ceil() as i32;
        grid.set_layout(nrows, ncols);

        let labels = vec!["Name", "Icon", "Command", "Arguments"];

        // Populates the grid
        let mut name_label = fltk::frame::Frame::default().with_label(labels[0]);
        let mut name_input = fltk::input::Input::default();
        name_input.set_value(grid_values[0]);
        grid.set_widget(&mut name_label, 0, 0).unwrap();
        grid.set_widget(&mut name_input, 0, 1).unwrap();

        let mut icon_label = fltk::frame::Frame::default().with_label(labels[1]);
        let icon_path = &config.assets_dir.join("generic.png");
        let image = fltk::image::PngImage::load(&icon_path).unwrap();
        let mut button_icon = fltk::button::Button::default().with_size(image.width(), image.height());
        button_icon.set_image(Some(image));

        // Usa un Rc per condividere lo stato tra il callback e il resto del codice
        let icon_path = Rc::new(RefCell::new(icon_path.clone()));
        let icon_path_clone = Rc::clone(&icon_path);

        let assets_dir = config.assets_dir.clone();
        let (w, h) = (config.icon_width, config.icon_height);
        button_icon.set_callback(move |b| {
            let mut chooser = fltk::dialog::FileChooser::new(
                &assets_dir,                    // directory
                "*.png",                    // filter or pattern
                fltk::dialog::FileChooserType::Single, // chooser type
                "Choose icon",     // title
            );
            chooser.show();
            while chooser.shown() {
                app::wait();
            }
            if !chooser.value(1).is_none() {
                let image_path = chooser.value(1).unwrap();
                let mut new_image = fltk::image::PngImage::load(&image_path).unwrap();
                new_image.scale(w, h, true, true);
                b.set_image(Some(new_image));
                *icon_path_clone.borrow_mut() = std::path::PathBuf::from(&image_path);
                b.redraw();
                let mut config = Ini::new();
                let tmp_file_path = crate::e4config::get_tmp_file();
                let result = config
                    .load(&tmp_file_path);
                config.set(crate::e4config::BUTTON_BUTTON_SECTION, "icon", Some(image_path));
                config.write(&tmp_file_path).expect("Cannot save the config file.");

                match result {
                    Ok(_) => (),
                    Err(e) => {
                        let message = format!("Cannot load the button config file: {}", e.to_string());
                        fltk::dialog::alert_default(&message);
                    },
                };
            }
        });

        grid.set_widget(&mut icon_label, 1, 0).unwrap();
        grid.set_widget(&mut button_icon, 1, 1).unwrap();

        let mut command_label = fltk::frame::Frame::default().with_label(labels[2]);
        let mut command_input = fltk::input::Input::default();
        command_input.set_value(grid_values[2]);
        grid.set_widget(&mut command_label, 2, 0).unwrap();
        grid.set_widget(&mut command_input, 2, 1).unwrap();

        let mut arguments_label = fltk::frame::Frame::default().with_label(labels[3]);
        let mut arguments_input = fltk::input::Input::default();
        arguments_input.set_value(command.get_arguments());
        grid.set_widget(&mut arguments_label, 3, 0).unwrap();
        grid.set_widget(&mut arguments_input, 3, 1).unwrap();

        let mut config_clone = config.clone();
        // Add OK button at the bottom
        let mut save_button = fltk::button::Button::new(200, 250, 100, 30, "Save");
        let sibling_name = sibling.name.clone();
        save_button.set_callback({
            let mut wind = wind.clone();
            move |_| {
                wind.hide();
                let tmp_file_path = crate::e4config::get_tmp_file();
                let mut tmp_config = Ini::new();
                let _ = tmp_config
                    .load(&tmp_file_path);
                let name = name_input.value();
                let mut config_file = config_clone.config_dir.join(&name);
                config_file.set_extension("conf");
                let command = command_input.value();
                let arguments = arguments_input.value();
                tmp_config.set(crate::e4config::BUTTON_BUTTON_SECTION, "command", Some(command));
                tmp_config.set(crate::e4config::BUTTON_BUTTON_SECTION, "arguments", Some(arguments));
                tmp_config.write(&tmp_file_path).expect(format!("Cannot save {}", &tmp_file_path.display()).as_str());
                std::fs::copy(tmp_file_path, &config_file).unwrap();
                // Modify e4docker.conf to put the button after sibling
                let number_of_buttons = config_clone.get_number_of_buttons() + 1;
                config_clone.set_number_of_buttons(number_of_buttons);
                let mut new_buttons = vec![];
                for button in &config_clone.buttons {
                    new_buttons.push(button.clone());
                    if button == &sibling_name {
                        new_buttons.push(name.to_string());
                    }
                }
                config_clone.set_buttons(&new_buttons);
                crate::e4config::restart_app();
            }
        });
        grid.set_widget(&mut save_button, 4, 0..2).unwrap();

        wind.make_modal(true);
        wind.end();
        wind.show();

        // Run modal window
        while wind.shown() {
            app::wait();
        }
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
                fltk::dialog::alert_default(&message);
            },
        };

        // Get the fields
        let command: String = config.get(crate::e4config::BUTTON_BUTTON_SECTION, "COMMAND").unwrap();
        let icon_path: String = config.get(crate::e4config::BUTTON_BUTTON_SECTION, "ICON").unwrap();
        let mut arguments: String = config.get(crate::e4config::BUTTON_BUTTON_SECTION, "ARGUMENTS").unwrap_or_else(|| {"".to_string()});
        arguments = arguments.trim().to_string();

        // Create the E4Command
        let command = E4Command::new(command, arguments);
        E4ButtonConfig { command, icon_path }
    }
}
