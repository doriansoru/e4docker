#![windows_subsystem = "windows"]
//! E4Docker - A simple docker for your favorite apps.
//!
//! Provided an docker for your favorite apps.
//! There are two main important directories:
//! - config: put here your e4docker.conf for the general configuration and a .conf file for each of your favorite apps.
//! - assets: put here the icons for your favourite apps.

use e4docker::{
    e4button::E4Button, e4config, e4config::E4Config, e4initialize, tr, translations::Translations,
};
use fltk::{app, enums, enums::FrameType, frame::Frame, menu, prelude::*, window::Window};
use round::round;
use std::{
    cell::RefCell,
    env,
    path::Path,
    rc::Rc,
    sync::{Arc, Mutex},
};

const APP_TITLE: &str = "E4 Docker";

fn about(translations: Arc<Mutex<Translations>>) {
    let version = env!("CARGO_PKG_VERSION");
    let authors = env!("CARGO_PKG_AUTHORS");
    e4config::create_about_dialog(
        &tr!(
            translations,
            format_display,
            "about-dialog",
            &[&version, &authors]
        ),
        translations.clone(),
    );
}

/// Redraw the [app] window.
fn redraw_window(
    project_config_dir: &Path,
    wind: &mut Window,
    translations: Arc<Mutex<Translations>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Read the global configuration
    let config = Rc::new(RefCell::new(E4Config::read(
        project_config_dir,
        translations.clone(),
    )?));
    let config_clone = config.clone();
    let config_other_clone = config.clone();

    let menu_height = round(config.borrow().window_height as f64 / 3.0, 0) as i32;
    wind.clear();
    wind.set_size(
        config.borrow().window_width,
        config.borrow().window_height + 2 * menu_height,
    );
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
    // Move the frame down to let space for the MenuBar
    frame.set_pos(frame.x(), frame.y() + menu_height);
    // Remove the border
    wind.set_border(false);

    // Put the buttons in the window
    let buttons = e4docker::e4button::create_buttons(
        &config.borrow(),
        wind,
        &frame,
        translations.clone(),
    );

    // For the menu bar
    let mut menubar = menu::MenuBar::default().with_size(config.borrow().window_width, menu_height);
    menubar.set_color(fltk::enums::Color::from_u32(0xe8dcca));
    menubar.set_frame(FrameType::FlatBox);
    let about_menu = match tr!(translations, get, "file-about-menu") {
        Some(m) => m.to_string(),
        None => "&File/About...\t".to_string(),
    };
    let quit_menu = match tr!(translations, get, "file-quit-menu") {
        Some(m) => m.to_string(),
        None => "&File/Quit\t".to_string(),
    };
    let translations_clone = translations.clone();
    let translations_second_clone = translations.clone();
    menubar.add(
        &about_menu,
        enums::Shortcut::Ctrl | 'a',
        menu::MenuFlag::Normal,
        move |_| {
            about(translations_clone.clone());
        },
    );
    menubar.add(
        &quit_menu,
        enums::Shortcut::Ctrl | 'q',
        menu::MenuFlag::Normal,
        |_| {
            app::quit();
        },
    );

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

    // For the popup menu
    let about_menu: &'static str =
        Box::leak(tr!(translations, get_or_default, "about", "About").into_boxed_str());
    let quit_menu: &'static str =
        Box::leak(tr!(translations, get_or_default, "quit", "Quit").into_boxed_str());
    let new_menu: &'static str =
        Box::leak(tr!(translations, get_or_default, "new", "New").into_boxed_str());
    let edit_menu: &'static str =
        Box::leak(tr!(translations, get_or_default, "edit-menu", "Edit").into_boxed_str());
    let delete_menu: &'static str =
        Box::leak(tr!(translations, get_or_default, "delete", "Delete").into_boxed_str());
    let empty_label_message = tr!(
        translations,
        get_or_default,
        "error-empty-menu-label",
        "Error: empty menu label"
    );
    let empty_label_message_clone = empty_label_message.clone();
    let empty_label_message_clone_bis = empty_label_message.clone();

    let menu = menu::MenuItem::new(&[about_menu, quit_menu]);

    let menu_clone = menu.clone();
    let menu_button = menu::MenuItem::new(&[new_menu, edit_menu, delete_menu]);
    let mut buttons_clone = buttons?.clone();

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
                        if (ex >= button.position.x && ex <= button.position.x + button.width)
                            && (ey >= button.position.y && ey <= button.position.y + button.height)
                            && button.button.active()
                        {
                            pressed_on_button = true;
                            if let Some(val) = menu_button.popup(ex, ey) {
                                match val.label() {
                                    Some(label) => {
                                        if label == new_menu {
                                            E4Button::add_button_after(
                                                &mut config.borrow_mut(),
                                                button,
                                                translations_second_clone.clone(),
                                            );
                                        } else if label == edit_menu {
                                            button.edit(
                                                &mut config.borrow_mut(),
                                                translations_second_clone.clone(),
                                            );
                                        } else if label == delete_menu {
                                            button.delete(
                                                &mut config.borrow_mut(),
                                                translations_second_clone.clone(),
                                            );
                                        }
                                    }
                                    None => {
                                        fltk::dialog::alert_default(&empty_label_message);
                                    }
                                }
                            }
                        }
                    }
                    if !pressed_on_button {
                        if let Some(val) = menu.popup(ex, ey) {
                            match val.label() {
                                Some(label) => {
                                    if label == about_menu {
                                        about(translations_second_clone.clone());
                                    } else if label == quit_menu {
                                        app::quit();
                                    }
                                }
                                None => {
                                    fltk::dialog::alert_default(&empty_label_message_clone);
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
            }
            // Handle the drag event
            enums::Event::Drag => {
                config_clone.borrow_mut().set_value(
                    e4config::E4DOCKER_DOCKER_SECTION.to_string(),
                    "x".to_string(),
                    Some((app::event_x_root() - x).to_string()),
                    translations_second_clone.clone(),
                );
                config_clone.borrow_mut().set_value(
                    e4config::E4DOCKER_DOCKER_SECTION.to_string(),
                    "y".to_string(),
                    Some((app::event_y_root() - y).to_string()),
                    translations_second_clone.clone(),
                );
                w.set_pos(app::event_x_root() - x, app::event_y_root() - y);
                true
            }
            _ => false,
        }
    });

    let mut wind_clone = wind.clone();
    menubar.handle({
        let mut x = 0;
        let mut y = 0;
        move |_, ev| match ev {
            enums::Event::Push => {
                // Handle the popup menu
                if app::event_mouse_button() == app::MouseButton::Right {
                    let (ex, ey) = app::event_coords();
                    if let Some(val) = menu_clone.popup(ex, ey) {
                        match val.label() {
                            Some(label) => {
                                if label == about_menu {
                                    about(translations.clone());
                                } else if label == quit_menu {
                                    app::quit();
                                }
                            }
                            None => {
                                fltk::dialog::alert_default(&empty_label_message_clone_bis);
                            }
                        }
                    }
                } else {
                    let coords = app::event_coords();
                    x = coords.0;
                    y = coords.1;
                }
                true
            }
            // Handle the drag event
            enums::Event::Drag => {
                config_other_clone.borrow_mut().set_value(
                    e4config::E4DOCKER_DOCKER_SECTION.to_string(),
                    "x".to_string(),
                    Some((app::event_x_root() - x).to_string()),
                    translations.clone(),
                );
                config_other_clone.borrow_mut().set_value(
                    e4config::E4DOCKER_DOCKER_SECTION.to_string(),
                    "y".to_string(),
                    Some((app::event_y_root() - y).to_string()),
                    translations.clone(),
                );
                wind_clone.set_pos(app::event_x_root() - x, app::event_y_root() - y);
                true
            }
            _ => false,
        }
    });

    Ok(())
}

fn main() {
    let translations = Translations::get_instance();
    // Get (or create) the path of the configuration directory for this app
    let project_config_dir = e4initialize::get_package_config_dir(translations.clone());

    // Create a FLTK app
    let app = app::App::default();

    // Create a window
    let mut wind = Window::default().with_label(APP_TITLE); //.center_screen();

    // Populate and draw the window
    match redraw_window(&project_config_dir, &mut wind, translations.clone()) {
        Ok(_) => {
            // Run the FLTK app
            match app.run() {
                Ok(_) => {}
                Err(e) => {
                    let message = tr!(translations, format_display, "cannot-exec-the-app", &[&e]);
                    fltk::dialog::alert_default(&message);
                }
            }
        }
        Err(e) => {
            let message = tr!(
                translations,
                format_display,
                "cannot-draw-the-window",
                &[&e]
            );
            fltk::dialog::alert_default(&message);
        }
    }
}
