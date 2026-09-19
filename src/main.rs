// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;
use std::process::Command;

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;

    ui.on_launch_app(move |app_name| {
        let script = format!("display dialog \"{}\"", app_name.replace('"', "\\\""));

        if let Err(error) = Command::new("osascript").args(["-e", &script]).spawn() {
            eprintln!("Could not launch {app_name}: {error}");
        }
    });

    let ui_handle = ui.as_weak();
    ui.on_quit(move || {
        ui_handle.unwrap().hide().expect("Could not close Farbig Launcher");
    });

    ui.run()?;

    Ok(())
}