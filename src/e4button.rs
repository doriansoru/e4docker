use crate::{
    e4command::E4Command, e4config::E4Config, e4icon::E4Icon, tr, translations::Translations,
};
use configparser::ini::Ini;
use fltk::{
    app, button::Button, enums::Color, frame::Frame, input::Input, prelude::*, window::Window,
};
use image::ImageReader;
use pelite::pe32::{Pe as Pe32, PeFile as PeFile32};
use pelite::pe64::{Pe as Pe64, PeFile as PeFile64};
use pelite::resources::Name;
use pelite::FileMap;
use round::round;
use std::{
    cell::RefCell,
    io::Cursor,
    path::PathBuf,
    rc::Rc,
    sync::{Arc, Mutex},
};

// The name of a generic E4Button: cannot be deleted
const GENERIC: &str = "generic";

/// The configuration for a [E4Button].
pub struct E4ButtonConfig {
    /// The [E4Command] containing the command and the args to exec.
    pub command: E4Command,
    /// The path of the [E4Icon] image for the [E4Button].
    pub icon_path: String,
}

/// Struct for the common ui between [E4Button::edit] and [E4Button::new_button]
struct E4ButtonEditUI {
    window: Window,
    name: Input,
    button_icon: Button,
    command: Input,
    command_button: Button,
    arguments: Input,
    save: Button,
}

impl E4ButtonEditUI {
    /// Create a ui and return the window, the inputs, the icon button and the save button
    fn new(translations: Arc<Mutex<Translations>>) -> Result<Self, Box<dyn std::error::Error>> {
        let mut window = Window::default().with_size(700, 300);
        let mut grid = fltk_grid::Grid::default()
            .with_size(650, 250)
            .center_of(&window);
        grid.show_grid(false);
        grid.set_gap(10, 10);
        let grid_values = ["", "", "", ""];
        // ncells = 10: Label and text for each value + Browse button + Save button
        let ncols = 3;
        let nrows = 5;
        grid.set_layout(nrows, ncols);

        let labels = [
            &tr!(translations, get_or_default, "name", "Name"),
            &tr!(translations, get_or_default, "icon", "Icon"),
            &tr!(translations, get_or_default, "command", "Command"),
            &tr!(translations, get_or_default, "arguments", "Arguments"),
        ];

        // Populates the grid
        let mut name_label = fltk::frame::Frame::default().with_label(labels[0]);
        let mut name_input = Input::default();
        name_input.set_value(grid_values[0]);
        grid.set_widget(&mut name_label, 0, 0)?;
        grid.set_widget(&mut name_input, 0, 1..3)?;

        let mut icon_label = fltk::frame::Frame::default().with_label(labels[1]);
        let mut button_icon = fltk::button::Button::default();

        grid.set_widget(&mut icon_label, 1, 0)?;
        grid.set_widget(&mut button_icon, 1, 1..3)?;

        let mut command_label = fltk::frame::Frame::default().with_label(labels[2]);
        let mut command_input = Input::default();
        let mut command_button = Button::default()
            .with_label(tr!(translations, get_or_default, "browse", "Browse").as_str());
        grid.set_widget(&mut command_label, 2, 0)?;
        grid.set_widget(&mut command_input, 2, 1)?;
        grid.set_widget(&mut command_button, 2, 2)?;

        let mut arguments_label = fltk::frame::Frame::default().with_label(labels[3]);
        let mut arguments_input = Input::default();
        grid.set_widget(&mut arguments_label, 3, 0)?;
        grid.set_widget(&mut arguments_input, 3, 1..3)?;

        // Add Save button at the bottom
        let mut save_button = fltk::button::Button::new(
            200,
            250,
            100,
            30,
            tr!(translations, get_or_default, "save", "Save").as_str(),
        );
        grid.set_widget(&mut save_button, 4, 0..3)?;

        window.make_modal(true);
        window.end();

        Ok(Self {
            window,
            name: name_input,
            button_icon,
            command: command_input,
            command_button,
            arguments: arguments_input,
            save: save_button,
        })
    }
}

/// A struct for the line below the [E4Button]
pub struct BorderIndicator {
    frame: Frame,
    is_active: bool,
}

impl std::clone::Clone for BorderIndicator {
    fn clone(&self) -> Self {
        Self {
            frame: self.frame.clone(),
            is_active: self.is_active,
        }
    }
}

impl BorderIndicator {
    fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        let mut frame = Frame::new(
            x,
            y + h + 2, // 2 pixel dal fondo
            w,
            2, // altezza della linea
            None,
        );
        frame.set_color(Color::White); // Inizialmente trasparente
        frame.set_frame(fltk::enums::FrameType::FlatBox);

        Self {
            frame,
            is_active: false,
        }
    }

    pub fn set_active(&mut self, active: bool) {
        if active != self.is_active {
            self.is_active = active;
            if active {
                self.frame.set_color(Color::Blue);
            } else {
                self.frame.set_color(Color::White);
            }
            self.frame.redraw();
        }
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }
}

/// A struct for the position of the button
pub struct Position {
    x: i32,
    y: i32,
}

impl Position {
    pub fn x(&self) -> i32 {
        self.x
    }

    pub fn y(&self) -> i32 {
        self.y
    }
}

impl std::clone::Clone for Position {
    fn clone(&self) -> Self {
        Self {
            x: self.x,
            y: self.y,
        }
    }
}

/// A struct for the sizze of the button
pub struct Size {
    pub w: i32,
    pub h: i32,
}

impl Size {
    pub fn new(w: i32, h: i32) -> Self {
        Self { w, h }
    }

    pub fn width(&self) -> i32 {
        self.w
    }

    pub fn height(&self) -> i32 {
        self.h
    }
}

impl std::clone::Clone for Size {
    fn clone(&self) -> Self {
        Self {
            w: self.w,
            h: self.h,
        }
    }
}

/// A fltk [Button] improved with a [E4Command].
pub struct E4Button {
    /// The name of the button, corresponding to the .conf file name
    pub name: String,
    /// The position of the button
    pub position: Position,
    /// The size of the button
    pub size: Size,
    /// The fltk [Button]
    pub button: Button,
    /// The button icon
    pub icon: E4Icon,
    /// The [E4Command] enclosed in a [Arc] to allow shared ownership and a [Mutex] to allow interior mutability through the [E4Button::set_command] implementation.
    pub command: Arc<Mutex<E4Command>>,
    /// The border of the [E4Button]
    pub border: BorderIndicator,
}

/// Create the [E4Button]s.
pub fn create_buttons(
    config: &E4Config,
    wind: &mut Window,
    frame: &Frame,
    translations: Arc<Mutex<Translations>>,
) -> Result<Vec<E4Button>, Box<dyn std::error::Error>> {
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
        let button_config: E4ButtonConfig =
            E4Button::read_config(config, button_name, translations.clone())?;
        // Create the icon
        let icon = E4Icon::new(
            PathBuf::from(button_config.icon_path),
            config.icon_width,
            config.icon_height,
        );
        // Create the command
        let command = Arc::new(Mutex::new(button_config.command));
        // Create the button
        current_e4button = E4Button::new(
            button_name,
            Position { x, y },
            frame,
            Arc::clone(&command),
            config,
            icon,
            translations.clone(),
        )?;
        current_e4button.button.set_tooltip(
            tr!(
                translations,
                format_display,
                "right-click-to-edit-delete-or-to-create-a-new-button-after",
                &[&button_name]
            )
            .as_str(),
        );
        // Add the button to the window
        wind.add(&current_e4button.button);
        buttons.push(current_e4button);
        x += config.icon_width + config.margin_between_buttons;
    }
    Ok(buttons)
}

/// Clone trait for [E4Button].
impl std::clone::Clone for E4Button {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            position: self.position.clone(),
            size: self.size.clone(),
            button: self.button.clone(),
            icon: self.icon.clone(),
            command: self.command.clone(),
            border: self.border.clone(),
        }
    }
}

impl E4Button {
    /// Transform the image to a fltk PngImage
    fn get_fltk_image(
        image_path: &PathBuf,
        translations: Arc<Mutex<Translations>>,
    ) -> Result<fltk::image::PngImage, Box<dyn std::error::Error>> {
        match &image_path.extension().and_then(std::ffi::OsStr::to_str) {
            Some(extension) => {
                let image_extension = extension.to_lowercase();
                let png_data = if image_extension != "exe" {
                    let new_image = ImageReader::open(image_path)?.decode()?;
                    let png_bytes: Vec<u8> = vec![];
                    let mut cursor = Cursor::new(png_bytes);
                    new_image.write_to(&mut cursor, image::ImageFormat::Png)?;
                    cursor.into_inner()
                } else {
                    // Open and map the exe file
                    match FileMap::open(image_path) {
                        Ok(file_map) => {
                            // Try as PE32
                            match PeFile32::from_bytes(&file_map) {
                                Ok(pe32) => {
                                    let resources = pe32.resources()?;
                                    // RT_ICON as Name::Id
                                    let icon = Name::Id(3); // RT_ICON

                                    // Get the first icon
                                    let icon_data =
                                        resources.find_resource(&[icon, Name::Id(1)])?;

                                    // Convert icon raw data to an image
                                    let img = image::load_from_memory(icon_data)?;

                                    // Prepare the buffer for the PNG
                                    let png_bytes: Vec<u8> = vec![];
                                    let mut cursor = Cursor::new(png_bytes);

                                    // Write the image as PNG
                                    img.write_to(&mut cursor, image::ImageFormat::Png)?;
                                    cursor.into_inner()
                                }
                                Err(_) => {
                                    // If PE32 fails, try as PE64
                                    match PeFile64::from_bytes(&file_map) {
                                        Ok(pe64) => {
                                            let resources = pe64.resources()?;
                                            // RT_ICON as Name::Id
                                            let icon = Name::Id(3); // RT_ICON

                                            // Get the first icon
                                            let icon_data =
                                                resources.find_resource(&[icon, Name::Id(1)])?;

                                            // Convert icon raw data to an image
                                            let img = image::load_from_memory(icon_data)?;

                                            // Prepare the buffer for the PNG
                                            let png_bytes: Vec<u8> = vec![];
                                            let mut cursor = Cursor::new(png_bytes);

                                            // Write the image as PNG
                                            img.write_to(&mut cursor, image::ImageFormat::Png)?;
                                            cursor.into_inner()
                                        }
                                        Err(e) => {
                                            // Cannot open the exe file. Return the generic icon
                                            let message = tr!(
                                                translations,
                                                format,
                                                "error-in-opening",
                                                &[
                                                    &image_path.display().to_string(),
                                                    &e.to_string()
                                                ]
                                            );
                                            fltk::dialog::alert_default(&message);
                                            let new_image = ImageReader::open(
                                                crate::e4initialize::get_generic_icon(Arc::clone(
                                                    &translations,
                                                )),
                                            )?
                                            .decode()?;
                                            let png_bytes: Vec<u8> = vec![];
                                            let mut cursor = Cursor::new(png_bytes);
                                            new_image
                                                .write_to(&mut cursor, image::ImageFormat::Png)?;
                                            cursor.into_inner()
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            let message = tr!(
                                translations,
                                format,
                                "error-in-opening",
                                &[&image_path.display().to_string(), &e.to_string()]
                            );
                            fltk::dialog::alert_default(&message);
                            vec![]
                        }
                    }
                };
                let fltk_image = if !png_data.is_empty() {
                    fltk::image::PngImage::from_data(&png_data)?
                } else {
                    let new_image = ImageReader::open(crate::e4initialize::get_generic_icon(
                        translations.clone(),
                    ))?
                    .decode()?;
                    let png_bytes: Vec<u8> = vec![];
                    let mut cursor = Cursor::new(png_bytes);
                    new_image.write_to(&mut cursor, image::ImageFormat::Png)?;
                    let png_data = cursor.into_inner();
                    fltk::image::PngImage::from_data(&png_data)?
                };
                Ok(fltk_image)
            }
            None => {
                let message = tr!(
                    translations,
                    format_display,
                    "error-in-getting-the-icon-extension",
                    &[&image_path.display()]
                );
                fltk::dialog::alert_default(&message);
                let new_image =
                    ImageReader::open(crate::e4initialize::get_generic_icon(translations.clone()))?
                        .decode()?;
                let png_bytes: Vec<u8> = vec![];
                let mut cursor = Cursor::new(png_bytes);
                new_image.write_to(&mut cursor, image::ImageFormat::Png)?;
                let png_data = cursor.into_inner();
                Ok(fltk::image::PngImage::from_data(&png_data)?)
            }
        }
    }

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
    /// use std::{sync::Arc, sync::Mutex, path::PathBuf};
    ///
    /// // Read the global configuration
    /// let directory = PathBuf::from("~")
    ///                     .join(".config")
    ///                     .join("e4docker")
    ///                     .join("config");
    /// let config = E4Config::read(&project_config_dir).unwrap();
    /// let frame = Frame::default();
    /// let command = E4Command::new(String::from("/usr/bin/nano"), vec![]);
    /// let command = Arc::new(Mutex::new(command));
    /// let icon = E4Icon::new(PathBuf::from("icon.png"), 64, 64);
    ///
    /// let my_button = E4Button::new(
    ///     name: "nano".to_string(),
    ///     position: Position { x: 0,
    ///     y: 0},
    ///     parent: &frame,
    ///     command,
    ///     config: &config,
    ///     icon,
    /// ).unwrap();
    /// ```
    pub fn new(
        name: &String,
        position: Position,
        parent: &Frame,
        command: Arc<Mutex<E4Command>>,
        config: &E4Config,
        icon: E4Icon,
        translations: Arc<Mutex<Translations>>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut button = Button::default()
            .with_pos(position.x, position.y)
            .with_size(icon.width(), icon.height())
            .center_y(parent);
        let (x, y) = (button.x(), button.y());
        let mut frame_border = Frame::new(
            button.x(),
            button.y() + button.height() - 2, // 2 pixel dal fondo
            button.width(),
            2, // altezza della linea
            None,
        );
        frame_border.set_color(Color::from_u32(0)); // Inizialmente trasparente
        frame_border.set_frame(fltk::enums::FrameType::FlatBox);

        let command_clone = Arc::clone(&command);
        let translations_clone = translations.clone();
        button.set_callback(move |_| {
            let mut guard = command_clone.lock().unwrap();
            let result = guard.exec();
            drop(guard);
            match result {
                Ok(_) => (),
                Err(e) => {
                    let guard = command_clone.lock().unwrap();
                    let message = tr!(
                        translations_clone,
                        format,
                        "failed-to-execute-command",
                        &[guard.get_cmd(), &e.to_string()]
                    );
                    drop(guard);
                    fltk::dialog::alert_default(&message);
                }
            };
        });

        // If the icon path does not exist, search for the icon in the assets directory
        let mut button_icon = if !icon.path().exists() {
            match Self::get_fltk_image(&config.assets_dir.join(icon.path()), translations.clone()) {
                Ok(image) => image,
                Err(e) => {
                    let message = tr!(
                        translations,
                        format,
                        "cannot-find",
                        &[
                            &config.assets_dir.join(icon.path()).display().to_string(),
                            &e.to_string()
                        ]
                    );
                    fltk::dialog::alert_default(&message);
                    let new_image = ImageReader::open(crate::e4initialize::get_generic_icon(
                        translations.clone(),
                    ))?
                    .decode()?;
                    let png_bytes: Vec<u8> = vec![];
                    let mut cursor = Cursor::new(png_bytes);
                    new_image.write_to(&mut cursor, image::ImageFormat::Png)?;
                    fltk::image::PngImage::from_data(&cursor.into_inner())?
                }
            }
        } else {
            match Self::get_fltk_image(icon.path(), translations.clone()) {
                Ok(image) => image,
                Err(e) => {
                    let message = tr!(
                        translations,
                        format,
                        "cannot-find",
                        &[
                            &config.assets_dir.join(icon.path()).display().to_string(),
                            &e.to_string()
                        ]
                    );
                    fltk::dialog::alert_default(&message);

                    let new_image = ImageReader::open(crate::e4initialize::get_generic_icon(
                        translations.clone(),
                    ))?
                    .decode()?;
                    let png_bytes: Vec<u8> = vec![];
                    let mut cursor = Cursor::new(png_bytes);
                    new_image.write_to(&mut cursor, image::ImageFormat::Png)?;
                    fltk::image::PngImage::from_data(&cursor.into_inner())?
                }
            }
        };
        let (w, h) = (icon.width(), icon.height());

        button_icon.scale(w, h, true, true);
        button.set_image(Some(button_icon));
        let border = BorderIndicator::new(x, y, w, h);
        Ok(E4Button {
            name: name.to_string(),
            position: Position { x, y },
            size: Size::new(w, y),
            button,
            icon,
            command,
            border,
        })
    }

    /// Set a new command for the [E4Button].
    pub fn set_command(&self, cmd: String, arguments: String) {
        let mut guard = self.command.lock().unwrap();
        guard.set(cmd, arguments);
        drop(guard);
    }

    /// Delete the [E4Button].
    pub fn delete(&mut self, config: &mut E4Config, translations: Arc<Mutex<Translations>>) {
        if self.name == GENERIC {
            let message = tr!(
                translations,
                get_or_default,
                "cannot-delete-the-generic-button",
                "Cannot delete the GENERIC button"
            );
            fltk::dialog::alert_default(&message);
            return;
        }

        // Delete the button configuration file
        let mut config_file = PathBuf::from(&self.name).with_extension("");
        config_file.set_extension("conf");
        config_file = config.config_dir.join(config_file);
        match std::fs::remove_file(&config_file) {
            Ok(_) => {}
            Err(e) => {
                let message = tr!(
                    translations,
                    format_display,
                    "cannot-remove-the-config-file",
                    &[&e]
                );
                fltk::dialog::alert_default(&message);
            }
        }

        // DON'T Delete the icon
        // self.icon.delete(&config);

        // Create a new buttons vec removing the one to be deleted
        let mut buttons = vec![];
        let old_buttons = config.buttons.clone();
        for (i, button) in old_buttons.iter().enumerate() {
            let button_number = i + 1;
            if *button != self.name {
                buttons.push(button.to_string());
            }
            let key_to_remove = format!("button{}", button_number);
            config.remove_key(
                crate::e4config::E4DOCKER_BUTTON_SECTION.to_string(),
                key_to_remove,
                translations.clone(),
            );
        }
        config.set_number_of_buttons(buttons.len() as i32, translations.clone());
        config.save_buttons(&buttons, translations.clone());
        crate::e4config::restart_app(translations.clone());
    }

    /// Edit the [E4Button].
    pub fn edit(&mut self, config: &mut E4Config, translations: Arc<Mutex<Translations>>) {
        // Create the ui
        match E4ButtonEditUI::new(translations.clone()) {
            Ok(mut ui) => {
                let mut config_file = config.config_dir.join(&self.name);
                config_file.set_extension("conf");
                let tmp_file_path = crate::e4config::get_tmp_file();
                match std::fs::copy(&config_file, &tmp_file_path) {
                    Ok(_) => {}
                    Err(e) => {
                        tr!(
                            translations,
                            format,
                            "cannot-copy-on",
                            &[
                                &config_file.display().to_string(),
                                &tmp_file_path.display().to_string(),
                                &e.to_string()
                            ]
                        );
                    }
                }
                ui.window
                    .set_label(tr!(translations, format, "edit", &[&self.name]).as_str());
                let command = self.command.lock().unwrap();
                let icon = self.icon.path().display().to_string();
                let grid_values = [
                    &self.name,
                    &icon,
                    &command.get_cmd().clone(),
                    &command.get_arguments().clone(),
                ];

                // Populate the ui
                ui.name.set_value(grid_values[0]);
                let icon_path = &config.assets_dir.join(self.icon.path());
                let mut image = match Self::get_fltk_image(icon_path, translations.clone()) {
                    Ok(img) => img,
                    Err(e) => {
                        panic!(
                            "{}",
                            tr!(
                                translations,
                                format,
                                "cannot-read-the-button-image",
                                &[&e.to_string()]
                            )
                        );
                    }
                };
                image.scale(self.size.width(), self.size.height(), true, true);
                ui.button_icon
                    .set_size(self.size.width(), self.size.height());

                ui.button_icon.set_image(Some(image));

                // Use an Rc to share the state between the callback and the rest of the code
                let icon_path = Rc::new(RefCell::new(icon_path.clone()));
                let icon_path_clone = Rc::clone(&icon_path);

                let assets_dir = config.assets_dir.clone();
                let (w, h) = (self.size.width(), self.size.height());
                let translations_clone = translations.clone();
                let translations_second_clone = translations.clone();
                let translations_third_clone = translations.clone();
                ui.button_icon.set_callback(move |b| {
                    let mut chooser = fltk::dialog::FileChooser::new(
                        &assets_dir,                           // directory
                        "*.png",                               // filter or pattern
                        fltk::dialog::FileChooserType::Single, // chooser type
                        &tr!(
                            translations_clone,
                            get_or_default,
                            "choose-icon",
                            "Choose icon"
                        ), // title
                    );
                    chooser.show();
                    while chooser.shown() {
                        app::wait();
                    }
                    if chooser.value(1).is_some() {
                        let image_path = match chooser.value(1) {
                            Some(img) => img,
                            None => panic!(
                                "{}",
                                tr!(
                                    translations,
                                    get_or_default,
                                    "cannot-find-the-chosen-image",
                                    "Cannot find the chosen image"
                                )
                            ),
                        };
                        let mut new_image = match Self::get_fltk_image(
                            &PathBuf::from(&image_path),
                            translations.clone(),
                        ) {
                            Ok(img) => img,
                            Err(e) => {
                                let message = tr!(
                                    translations,
                                    format,
                                    "cannot-load-the-image",
                                    &[&e.to_string()]
                                );
                                fltk::dialog::alert_default(&message);
                                match Self::get_fltk_image(
                                    &icon_path_clone.borrow_mut(),
                                    translations.clone(),
                                ) {
                                    Ok(img) => img,
                                    Err(e) => {
                                        panic!(
                                            "{}",
                                            tr!(
                                                translations,
                                                format,
                                                "cannot-read-the-button-image",
                                                &[&e.to_string()]
                                            )
                                        );
                                    }
                                }
                            }
                        };
                        new_image.scale(w, h, true, true);
                        b.set_image(Some(new_image));
                        *icon_path_clone.borrow_mut() = std::path::PathBuf::from(&image_path);
                        b.redraw();
                        let mut config = Ini::new();
                        let tmp_file_path = crate::e4config::get_tmp_file();
                        let result = config.load(&tmp_file_path);
                        config.set(
                            crate::e4config::BUTTON_BUTTON_SECTION,
                            "icon",
                            Some(image_path),
                        );
                        config.write(&tmp_file_path).expect(&tr!(
                            translations,
                            get_or_default,
                            "cannot-save-the-config-file",
                            "Cannot save the config file"
                        ));

                        match result {
                            Ok(_) => (),
                            Err(e) => {
                                let message = tr!(
                                    translations,
                                    format_display,
                                    "cannot-load-the-button-config-file",
                                    &[&e]
                                );
                                fltk::dialog::alert_default(&message);
                            }
                        };
                    }
                });

                ui.command.set_value(grid_values[2]);
                let mut command_clone = ui.command.clone();

                ui.command_button.set_callback(move |_| {
                    // Obtain the current directory
                    let current_dir = match std::env::current_dir() {
                        Ok(dir) => dir,
                        Err(e) => {
                            panic!(
                                "{}",
                                tr!(
                                    translations_second_clone,
                                    format,
                                    "cannot-get-che-current-directory",
                                    &[&e.to_string()]
                                )
                            );
                        }
                    };

                    // Go up until the root directory
                    let mut root_dir = current_dir;
                    while let Some(parent) = root_dir.parent() {
                        root_dir = parent.to_path_buf();
                    }

                    let mut chooser = fltk::dialog::FileChooser::new(
                        &root_dir,                             // directory
                        "*",                                   // filter or pattern
                        fltk::dialog::FileChooserType::Single, // chooser type
                        &tr!(
                            translations_second_clone,
                            get_or_default,
                            "choose-a-program",
                            "Choose a program"
                        ), // title
                    );
                    chooser.show();
                    while chooser.shown() {
                        app::wait();
                    }
                    if chooser.value(1).is_some() {
                        let command_path = match chooser.value(1) {
                            Some(cmd) => cmd,
                            None => panic!(
                                "{}",
                                tr!(
                                    translations_second_clone,
                                    get_or_default,
                                    "cannot-find-the-chosen-command",
                                    "Cannot find the chosen command"
                                )
                            ),
                        };
                        command_clone.set_value(&command_path);
                    }
                });

                ui.arguments.set_value(command.get_arguments());
                drop(command);
                // Add OK button at the bottom
                let mut config_clone = config.clone();
                let old_name = self.name.clone();

                ui.save.set_callback({
                    let mut wind = ui.window.clone();
                    move |_| {
                        wind.hide();
                        let tmp_file_path = crate::e4config::get_tmp_file();
                        let mut tmp_config = Ini::new();
                        let _ = tmp_config.load(&tmp_file_path);
                        let name = ui.name.value();
                        if name == GENERIC {
                            let message = tr!(
                                translations_third_clone,
                                get_or_default,
                                "cannot-modify-the-generic-button",
                                "Cannot modify the GENERIC button"
                            );
                            fltk::dialog::alert_default(&message);
                            return;
                        }
                        let mut config_file = config_clone.config_dir.join(&name);
                        config_file.set_extension("conf");
                        let command = ui.command.value();
                        let arguments = ui.arguments.value();
                        tmp_config.set(
                            crate::e4config::BUTTON_BUTTON_SECTION,
                            "command",
                            Some(command),
                        );
                        tmp_config.set(
                            crate::e4config::BUTTON_BUTTON_SECTION,
                            "arguments",
                            Some(arguments),
                        );
                        match tmp_config.write(&tmp_file_path) {
                            Ok(_) => {}
                            Err(e) => {
                                panic!(
                                    "{}",
                                    tr!(
                                        translations_third_clone,
                                        format,
                                        "cannot-save",
                                        &[&tmp_file_path.display().to_string(), &e.to_string()]
                                    )
                                );
                            }
                        }
                        let mut n = 0;
                        for (i, button) in config_clone.buttons.iter().enumerate() {
                            if *button == old_name {
                                n = i + 1;
                            }
                        }
                        config_clone.set_value(
                            crate::e4config::E4DOCKER_BUTTON_SECTION.to_string(),
                            format!("button{}", n),
                            Some(name),
                            translations_third_clone.clone(),
                        );
                        match std::fs::copy(&tmp_file_path, &config_file) {
                            Ok(_) => {}
                            Err(e) => {
                                panic!(
                                    "{}",
                                    tr!(
                                        translations_third_clone,
                                        format,
                                        "cannot-copy-the-temporary-file-to-the-config-file",
                                        &[
                                            &tmp_file_path.display().to_string(),
                                            &config_file.display().to_string(),
                                            &e.to_string()
                                        ]
                                    )
                                );
                            }
                        }
                        crate::e4config::restart_app(translations_third_clone.clone());
                    }
                });

                ui.window.show();

                // Run modal window
                while ui.window.shown() {
                    app::wait();
                }
            }
            Err(e) => {
                let message = tr!(
                    translations,
                    format,
                    "cannot-get-the-buttons-ui",
                    &[&e.to_string()]
                );
                fltk::dialog::alert_default(&message);
            }
        }
    }

    /// Create a new [E4Button] at the end.
    pub fn new_button(config: &mut E4Config, translations: Arc<Mutex<Translations>>) {
        match E4ButtonEditUI::new(translations.clone()) {
            Ok(mut ui) => {
                let name = GENERIC;
                let mut config_file = config.config_dir.join(name);
                config_file.set_extension("conf");
                let tmp_file_path = crate::e4config::get_tmp_file();

                match std::fs::copy(&config_file, &tmp_file_path) {
                    Ok(_) => {}
                    Err(e) => {
                        panic!(
                            "{}",
                            tr!(
                                translations,
                                format,
                                "cannot-copy-the-on",
                                &[
                                    &config_file.display().to_string(),
                                    &tmp_file_path.display().to_string(),
                                    &e.to_string()
                                ]
                            )
                        );
                    }
                }
                let button_config =
                    match Self::read_config(config, &name.to_string(), translations.clone()) {
                        Ok(config) => config,
                        Err(e) => {
                            panic!(
                                "{}",
                                tr!(
                                    translations,
                                    format,
                                    "cannot-read-the-generic-button-configuration-file",
                                    &[&e.to_string()]
                                )
                            );
                        }
                    };
                ui.window.set_label(&tr!(
                    translations,
                    get_or_default,
                    "new-button",
                    "New Button"
                ));
                let command = button_config.command;
                let icon = button_config.icon_path;
                let grid_values = [name, &icon, command.get_cmd(), command.get_arguments()];

                // Populate the ui
                ui.name.set_value(grid_values[0]);

                let icon_path = &mut config.assets_dir.join(GENERIC);
                icon_path.set_extension("png");
                let image = match Self::get_fltk_image(icon_path, translations.clone()) {
                    Ok(img) => img,
                    Err(e) => panic!(
                        "{}",
                        tr!(
                            translations,
                            format,
                            "cannot-get",
                            &[&icon_path.display().to_string(), &e.to_string()]
                        )
                    ),
                };
                ui.button_icon.set_image(Some(image));

                // Use a Rc to share the state between the callback and the rest of the code
                let icon_path = Rc::new(RefCell::new(icon_path.clone()));
                let icon_path_clone = Rc::clone(&icon_path);

                let assets_dir = config.assets_dir.clone();
                let (w, h) = (config.icon_width, config.icon_height);
                let translations_clone = translations.clone();
                let translations_second_clone = translations.clone();
                let translations_third_clone = translations.clone();
                ui.button_icon.set_callback(move |b| {
                    let mut chooser = fltk::dialog::FileChooser::new(
                        &assets_dir,                           // directory
                        "*.png",                               // filter or pattern
                        fltk::dialog::FileChooserType::Single, // chooser type
                        &tr!(
                            translations_clone,
                            get_or_default,
                            "choose-icon",
                            "Choose icon"
                        ), // title
                    );
                    chooser.show();
                    while chooser.shown() {
                        app::wait();
                    }
                    if chooser.value(1).is_some() {
                        let image_path = match chooser.value(1) {
                            Some(img) => img,
                            None => panic!(
                                "{}",
                                tr!(
                                    translations,
                                    get_or_default,
                                    "cannot-find-the-chosen-image",
                                    "Cannot find the chosen image"
                                )
                            ),
                        };
                        let mut new_image = match Self::get_fltk_image(
                            &PathBuf::from(&image_path),
                            translations.clone(),
                        ) {
                            Ok(img) => img,
                            Err(e) => {
                                let message = tr!(
                                    translations,
                                    format,
                                    "cannot-load-the-image",
                                    &[&e.to_string()]
                                );
                                fltk::dialog::alert_default(&message);
                                match Self::get_fltk_image(
                                    &icon_path_clone.borrow_mut(),
                                    translations.clone(),
                                ) {
                                    Ok(img) => img,
                                    Err(e) => {
                                        panic!(
                                            "{}",
                                            tr!(
                                                translations,
                                                format,
                                                "cannot-read-the-button-image",
                                                &[&e.to_string()]
                                            )
                                        );
                                    }
                                }
                            }
                        };
                        new_image.scale(w, h, true, true);
                        b.set_image(Some(new_image));
                        *icon_path_clone.borrow_mut() = std::path::PathBuf::from(&image_path);
                        b.redraw();
                        let mut config = Ini::new();
                        let tmp_file_path = crate::e4config::get_tmp_file();
                        let result = config.load(&tmp_file_path);
                        config.set(
                            crate::e4config::BUTTON_BUTTON_SECTION,
                            "icon",
                            Some(image_path),
                        );
                        config.write(&tmp_file_path).expect(&tr!(
                            translations,
                            get_or_default,
                            "cannot-save-the-config-file",
                            "Cannot save the config file"
                        ));

                        match result {
                            Ok(_) => (),
                            Err(e) => {
                                let message = tr!(
                                    translations,
                                    format,
                                    "cannot-load-the-button-config-file",
                                    &[&e.to_string()]
                                );
                                fltk::dialog::alert_default(&message);
                            }
                        };
                    }
                });

                ui.command.set_value(grid_values[2]);
                let mut command_clone = ui.command.clone();
                ui.command_button.set_callback(move |_| {
                    // Ottieni la directory corrente
                    let current_dir = match std::env::current_dir() {
                        Ok(dir) => dir,
                        Err(e) => {
                            panic!(
                                "{}",
                                tr!(
                                    translations_second_clone,
                                    format,
                                    "cannot-get-che-current-directory",
                                    &[&e.to_string()]
                                )
                            );
                        }
                    };

                    // Risali fino alla radice
                    let mut root_dir = current_dir;
                    while let Some(parent) = root_dir.parent() {
                        root_dir = parent.to_path_buf();
                    }

                    let mut chooser = fltk::dialog::FileChooser::new(
                        &root_dir,                             // directory
                        "*",                                   // filter or pattern
                        fltk::dialog::FileChooserType::Single, // chooser type
                        &tr!(
                            translations_second_clone,
                            get_or_default,
                            "choose-a-program",
                            "Choose a program"
                        ), // title
                    );

                    chooser.show();
                    while chooser.shown() {
                        app::wait();
                    }
                    if chooser.value(1).is_some() {
                        let command_path = match chooser.value(1) {
                            Some(cmd) => cmd,
                            None => panic!(
                                "{}",
                                tr!(
                                    translations_second_clone,
                                    get_or_default,
                                    "cannot-find-the-chosen-command",
                                    "Cannot find the chosen command"
                                )
                            ),
                        };
                        command_clone.set_value(&command_path);
                    }
                });

                ui.arguments.set_value(command.get_arguments());

                let mut config_clone = config.clone();
                // Add OK button at the bottom
                ui.save.set_callback({
                    let mut wind = ui.window.clone();
                    move |_| {
                        wind.hide();
                        let tmp_file_path = crate::e4config::get_tmp_file();
                        let mut tmp_config = Ini::new();
                        let _ = tmp_config.load(&tmp_file_path);
                        let name = ui.name.value();
                        let mut config_file = config_clone.config_dir.join(&name);
                        config_file.set_extension("conf");
                        let command = ui.command.value();
                        let arguments = ui.arguments.value();
                        tmp_config.set(
                            crate::e4config::BUTTON_BUTTON_SECTION,
                            "command",
                            Some(command),
                        );
                        tmp_config.set(
                            crate::e4config::BUTTON_BUTTON_SECTION,
                            "arguments",
                            Some(arguments),
                        );
                        match tmp_config.write(&tmp_file_path) {
                            Ok(_) => {}
                            Err(e) => {
                                panic!(
                                    "{}",
                                    tr!(
                                        translations_third_clone,
                                        format,
                                        "cannot-save",
                                        &[&tmp_file_path.display().to_string(), &e.to_string()]
                                    )
                                );
                            }
                        }

                        match std::fs::copy(&tmp_file_path, &config_file) {
                            Ok(_) => {}
                            Err(e) => {
                                panic!(
                                    "{}",
                                    tr!(
                                        translations_third_clone,
                                        format,
                                        "cannot-copy-the-on",
                                        &[
                                            &tmp_file_path.display().to_string(),
                                            &config_file.display().to_string(),
                                            &e.to_string()
                                        ]
                                    )
                                );
                            }
                        };

                        // Modify the number of buttons and the buttons list in e4docker.conf.
                        let number_of_buttons = match config_clone
                            .get_number_of_buttons(translations_third_clone.clone())
                        {
                            Ok(b) => b + 1,
                            Err(e) => {
                                panic!(
                                    "{}",
                                    tr!(
                                        translations_third_clone,
                                        format,
                                        "cannot-get-the-number-of-buttons",
                                        &[&e.to_string()]
                                    )
                                );
                            }
                        };
                        config_clone.set_number_of_buttons(
                            number_of_buttons,
                            translations_third_clone.clone(),
                        );
                        let mut new_buttons = vec![];
                        for button in &config_clone.buttons {
                            new_buttons.push(button.clone());
                        }
                        new_buttons.push(name.to_string());
                        config_clone.save_buttons(&new_buttons, translations_third_clone.clone());
                        crate::e4config::restart_app(translations_third_clone.clone());
                    }
                });

                ui.window.show();

                // Run modal window
                while ui.window.shown() {
                    app::wait();
                }
            }
            Err(e) => {
                let message = tr!(
                    translations,
                    format,
                    "cannot-get-the-buttons-ui",
                    &[&e.to_string()]
                );
                fltk::dialog::alert_default(&message);
            }
        }
    }

    /// Read the configuration of a [E4Button] from confi/button_name.conf.
    /// Return an instance of a [E4ButtonConfig], containing the fltk [Button] and the [E4Command].
    pub fn read_config(
        config: &E4Config,
        button_name: &String,
        translations: Arc<Mutex<Translations>>,
    ) -> Result<E4ButtonConfig, Box<dyn std::error::Error>> {
        // Read config.config_dir/button_name.conf
        let mut config_file = config.config_dir.join(button_name);
        config_file.set_extension("conf");
        let mut config = Ini::new();
        let result = config.load(config_file);

        match result {
            Ok(_) => (),
            Err(e) => {
                let message = tr!(
                    translations,
                    format,
                    "cannot-load-the-button-config-file",
                    &[&e.to_string()]
                );
                fltk::dialog::alert_default(&message);
            }
        };

        // Get the fields
        let command: String = match config.get(crate::e4config::BUTTON_BUTTON_SECTION, "COMMAND") {
            Some(command) => command,
            None => "".to_string(),
        };
        let icon_path: String = match config.get(crate::e4config::BUTTON_BUTTON_SECTION, "ICON") {
            Some(path) => path,
            None => crate::e4initialize::get_generic_icon(translations.clone())
                .display()
                .to_string(),
        };
        let mut arguments: String =
            match config.get(crate::e4config::BUTTON_BUTTON_SECTION, "ARGUMENTS") {
                Some(arg) => arg,
                None => "".to_string(),
            };
        arguments = arguments.trim().to_string();

        // Create the E4Command
        let command = E4Command::new(command, arguments);
        Ok(E4ButtonConfig { command, icon_path })
    }
}
