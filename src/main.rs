//! E4Docker - A simple docker for your favorite apps.
//!
//! Provided an docker for your favorite apps.
//! There are two main important directories:
//! - config: put here your e4docker.conf for the general configuration and a .conf file for each of your favorite apps.
//! - assets: put here the icons for your favourite apps.

use e4docker::{e4button::E4Button, e4button::E4ButtonConfig, e4config::E4Config, e4icon::E4Icon};
use fltk::{app, enums, enums::FrameType, frame::Frame, menu, prelude::*, window::Window};
use round::round;
use std::{cell::RefCell, env, fs, path::PathBuf, rc::Rc};

const APP_TITLE: &str = "E4 Docker";

fn main() {
    // Get the package name
    let package_name = env!("CARGO_PKG_NAME");

    // Get the user config dir
    let config_dir = dirs::config_dir().expect("Cannot find your config dir.");
    // Create the path of the configuration directory for this app
    let project_config_dir = config_dir.join(package_name);
    // Create this app configuration directory if it does not exist
    if !project_config_dir.exists() {
        fs::create_dir_all(&project_config_dir)
            .expect("Cannot create the project config directory.");
    }

    // Read the global configuration
    let config = E4Config::read(&project_config_dir);

    // Create a FLTK app
    let app = app::App::default();

    // Create a window
    let mut wind = Window::default()
        .with_size(config.window_width, config.window_height)
        .center_screen()
        .with_label(APP_TITLE);
    // Create a frame
    let mut frame = Frame::default()
        .with_size(
            config.window_width - config.frame_margin,
            config.window_height - config.frame_margin,
        )
        .center_of(&wind)
        .center_of(&wind)
        .with_label("");
    frame.set_frame(FrameType::EngravedBox);
    // Remove the border
    wind.set_border(false);
    wind.end();
    wind.show();
    // Always on top
    wind.set_on_top();

    // For the popup menu
    let mut menu = menu::MenuItem::new(&["Quit"]);
    let mut menu_label_size: i32 = round(app::screen_size().0 / 100.0, 0) as i32;
    if menu_label_size < 15 { menu_label_size = 15; };
    menu.set_label_size(menu_label_size);

    // Handle tre popup menu and the drag event
    wind.handle({
        let mut x = 0;
        let mut y = 0;
        move |w, ev| match ev {
            enums::Event::Push => {
                // Handle the popup menu
                if app::event_mouse_button() == app::MouseButton::Right {
                    let (ex, ey) = app::event_coords();
                    match menu.popup(ex, ey) {
                        Some(_val) => app::quit(),
                        None => {},
                    }
                } else {
                    let coords = app::event_coords();
                    x = coords.0;
                    y = coords.1;
                }
                true
            },
            // Handle the drag event
            enums::Event::Drag => {
                w.set_pos(app::event_x_root() - x, app::event_y_root() - y);
                true
            }
            _ => false,
        }
    });

    // Put the buttons in the window
    let mut x = config.margin_between_buttons;
    let y: i32 = round(
        (config.window_height as f64 - config.icon_height as f64) / 2.0,
        0,
    ) as i32;
    for button_name in &config.buttons {
        // Read the button config
        let button_config: E4ButtonConfig = E4Button::read_config(&config, button_name);
        // Create the icon
        let icon = E4Icon::new(
            PathBuf::from(button_config.icon_path),
            config.icon_width,
            config.icon_height,
        );
        // Create the command
        let command = Rc::new(RefCell::new(button_config.command));
        // Create the button
        let current_e4button = E4Button::new(
            x,
            y,
            &frame,
            Rc::clone(&command),
            &config,
            icon,
        );
        // Add the button to the window
        wind.add(&current_e4button.button);
        x += config.icon_width + config.margin_between_buttons;
    }

    // Run the FLTK app
    app.run().expect("Cannot exec the app.");
}
