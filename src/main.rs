//! E4Docker - A simple docker for your favorite apps.
//!
//! Provided an docker for your favorite apps.
//! There are two main important directories:
//! - config: put here your e4docker.conf for the general configuration and a .conf file for each of your favorite apps.
//! - assets: put here the icons for your favourite apps.

use e4docker::{e4button::E4Button, e4button::E4ButtonConfig, e4config::E4Config, e4icon::E4Icon};
use fltk::{app, enums, enums::FrameType, frame::Frame, menu, prelude::*,window::Window};
use round::round;
use std::{cell::RefCell, env, fs, path::PathBuf, rc::Rc};

const APP_TITLE: &str = "E4 Docker";

fn create_about_dialog(message: &str) {
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
    let config = Rc::new(RefCell::new(E4Config::read(&project_config_dir)));
    let config_clone = config.clone();

    // Create a FLTK app
    let app = app::App::default();

    // Create a window
    let mut wind = Window::default()
        .with_size(config.borrow().window_width, config.borrow().window_height)
        .with_label(APP_TITLE);//.center_screen();

    // Create a frame
    let mut frame = Frame::default()
        .with_size(
            config.borrow().window_width - config.borrow().frame_margin,
            config.borrow().window_height - config.borrow().frame_margin,
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

    let cx: i32 = config.borrow().x;
    let cy: i32 = config.borrow().y;

    if cx != 0 {
        let _ = &wind.set_pos(cx, cy);
    }

    // For the popup menu
    let menu = menu::MenuItem::new(&["About", "Quit"]);

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
                        Some(val) => {
                            let label = val.label().unwrap();
                            match label.as_str() {
                                "About" => {
                                    create_about_dialog("E4Docker 0.1.0.\nBy Dorian Soru.\n<doriansoru@gmail.com>\nReleased in 2024.");
                                }
                                "Quit" => {
                                    app::quit();
                                }
                                _ => {
                                    
                                }
                            }
                        },
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
                config_clone.borrow_mut().save_value("E4DOCKER".to_string(), "x".to_string(), (app::event_x_root() - x).to_string());
                config_clone.borrow_mut().save_value("E4DOCKER".to_string(), "y".to_string(), (app::event_y_root() - y).to_string());
                w.set_pos(app::event_x_root() - x, app::event_y_root() - y);
                true
            }
            _ => false,
        }
    });

    // Put the buttons in the window
    let mut x = config.borrow().margin_between_buttons;
    let y: i32 = round(
        (config.borrow().window_height as f64 - config.borrow().icon_height as f64) / 2.0,
        0,
    ) as i32;
    for button_name in &config.borrow().buttons {
        // Read the button config
        let button_config: E4ButtonConfig = E4Button::read_config(&config.borrow(), &button_name);
        // Create the icon
        let icon = E4Icon::new(
            PathBuf::from(button_config.icon_path),
            config.borrow().icon_width,
            config.borrow().icon_height,
        );
        // Create the command
        let command = Rc::new(RefCell::new(button_config.command));
        // Create the button
        let current_e4button = E4Button::new(
            x,
            y,
            &frame,
            Rc::clone(&command),
            &config.borrow(),
            icon,
        );
        // Add the button to the window
        wind.add(&current_e4button.button);
        x += config.borrow().icon_width + config.borrow().margin_between_buttons;
    }

    // Run the FLTK app
    app.run().expect("Cannot exec the app.");
}
