use crate::e4button::E4Button;
use fltk::app;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use sysinfo::System;

/// Check if a process is running by using sysinfo
fn is_process_running(sys: &System, process_path: &str) -> bool {
    // Extract the file name from the full path
    let process_name = Path::new(process_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(process_path);

    // Search among all processes
    sys.processes().values().any(|process| {
        // Compare both the full path and the file name
        process.name().to_str().unwrap().contains(process_name)
            || process
                .cmd()
                .iter()
                .any(|cmd| cmd.to_str().unwrap().contains(process_name))
    })
}

/// Start a thread to check periodically all processes
pub fn start_process_checker(buttons: Arc<Mutex<Vec<E4Button>>>, app: &app::App) {
    let interval = 2;
    // Modifichiamo il channel per inviare l'indice invece del riferimento al button
    let (sender, receiver) = app::channel::<(usize, bool)>();
    let app_clone = *app;

    let buttons_for_thread = buttons.clone();

    thread::spawn(move || {
        let mut sys = System::new_all();
        loop {
            sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

            let buttons = buttons_for_thread.lock().unwrap();
            for (index, button) in buttons.iter().enumerate() {
                let cmd = button.command.lock().unwrap();
                let is_running = is_process_running(&sys, cmd.get());
                sender.send((index, is_running));
            }
            drop(buttons);

            thread::sleep(Duration::from_secs(interval));
        }
    });

    while app_clone.wait() {
        if let Some((index, is_running)) = receiver.recv() {
            let mut buttons = buttons.lock().unwrap();
            if let Some(button) = buttons.get_mut(index) {
                button.border.set_active(is_running);
            }
        }
    }
}

/// Setup of the process checker
pub fn setup_process_checker(buttons: Vec<E4Button>, app: &app::App) {
    let buttons = Arc::new(Mutex::new(buttons));
    start_process_checker(buttons.clone(), app);
}
