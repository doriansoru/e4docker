#![windows_subsystem = "windows"]
//! E4Docker - A simple docker for your favorite apps.
//!
//! Provided an docker for your favorite apps.
//! There are two main important directories:
//! - config: put here your e4docker.conf for the general configuration and a .conf file for each of your favorite apps.
//! - assets: put here the icons for your favourite apps.

use e4docker::{e4button::E4Button, e4config, e4config::E4Config, e4initialize};
use fltk::{app, enums, enums::FrameType, frame::Frame, menu, prelude::*,window::Window};
use std::{cell::RefCell, env, path::Path, rc::Rc};

const APP_TITLE: &str = "E4 Docker";

/// Redraw the [app] window.
fn redraw_window(project_config_dir: &Path, wind: &mut Window) -> Vec<E4Button> {
    // Read the global configuration
    let config = Rc::new(RefCell::new(E4Config::read(project_config_dir)));
    let config_clone = config.clone();

    wind.clear();
    wind.set_size(config.borrow().window_width, config.borrow().window_height);
    // Create a frame
    let mut frame = Frame::default()
        .with_size(
            config.borrow().window_width - config.borrow().frame_margin,
            config.borrow().window_height - config.borrow().frame_margin,
        )
        //.center_of(&wind)
        .center_of(wind)
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
        //let _ = &wind.set_pos(cx, cy);
        wind.set_pos(cx, cy);
    }

    // Put the buttons in the window
    let buttons = e4docker::e4button::create_buttons(&config.borrow(), wind, &frame);

    // For the popup menu
    let menu = menu::MenuItem::new(&["About", "Quit"]);
    let menu_button = menu::MenuItem::new(&["New", "Edit", "Delete"]);

    let mut buttons_clone = buttons.clone();

    // Handle tre popup menu and the drag event
    wind.handle({
        let mut x = 0;
        let mut y = 0;
        move |w, ev| match ev {
            enums::Event::Push => {
                // Handle the popup menu
                if app::event_mouse_button() == app::MouseButton::Right {
                    let mut pressed_on_button: bool = false;
                    let (ex, ey) = app::event_coords();
                    for button in &mut buttons_clone {
                        if (ex >= button.x && ex <= button.x + button.width) &&
                        (ey >= button.y && ey <= button.y + button.height) && button.button.active() {
                            pressed_on_button = true;
                            if let Some(val) = menu_button.popup(ex, ey) {
                                let label = val.label().unwrap();
                                match label.as_str() {
                                    "New" => {
                                        E4Button::new_button(&mut config.borrow_mut(), button);
                                    },
                                    "Edit" => {
                                        button.edit(&mut config.borrow_mut());
                                    },
                                    "Delete" => {
                                        button.delete(&mut config.borrow_mut());
                                    },
                                    _ => {

                                    }
                                }
                            }
                        }
                    }
                    if !pressed_on_button {
                        if let Some(val) = menu.popup(ex, ey) {
                            let label = val.label().unwrap();
                            match label.as_str() {
                                "About" => {
                                    let version = env!("CARGO_PKG_VERSION");
                                    let authors = env!("CARGO_PKG_AUTHORS");
                                    e4config::create_about_dialog(format!("E4Docker {}.\nBy {}\nReleased in 2024.", version, authors).as_str());
                                },
                                "Quit" => {
                                    app::quit();
                                },
                                _ => {

                                }
                            }
                        }
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
                config_clone.borrow_mut().set_value(e4config::E4DOCKER_DOCKER_SECTION.to_string(), "x".to_string(), Some((app::event_x_root() - x).to_string()));
                config_clone.borrow_mut().set_value(e4config::E4DOCKER_DOCKER_SECTION.to_string(), "y".to_string(), Some((app::event_y_root() - y).to_string()));
                w.set_pos(app::event_x_root() - x, app::event_y_root() - y);
                true
            }
            _ => false,
        }
    });
    buttons
}

fn main() {
    // Get (or create) the path of the configuration directory for this app
    let project_config_dir = e4initialize::get_package_config_dir();

    // Create a FLTK app
    let app = app::App::default();

    // Create a window
    let mut wind = Window::default()
        .with_label(APP_TITLE);//.center_screen();

    // Populate and draw the window
    let _ = redraw_window(&project_config_dir, &mut wind);

    // Run the FLTK app
    app.run().expect("Cannot exec the app.");
}
