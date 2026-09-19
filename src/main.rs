// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;
use std::fs;
use std::process::Command;

use slint::{ModelRc, SharedString, VecModel};

slint::include_modules!();

fn apps() -> Result<Vec<App>, std::io::Error> {
    let mut bundles = fs::read_dir("/Applications")?
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            (path.extension().is_some_and(|extension| extension == "app")).then_some(path)
        })
        .filter_map(|path| {
            let name = path.file_stem()?.to_string_lossy().into_owned();
            let path = path.to_string_lossy().into_owned();

            Some(App {
                name: SharedString::from(name),
                path: SharedString::from(path),
            })
        })
        .collect::<Vec<_>>();

    bundles.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(bundles)
}

fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;
    ui.set_apps(ModelRc::new(VecModel::from(apps()?)));

    ui.on_launch_app(move |app_path| {
        if let Err(error) = Command::new("open").arg(app_path.as_str()).spawn() {
            eprintln!("Could not launch {app_path}: {error}");
        }
    });

    let ui_handle = ui.as_weak();
    ui.on_quit(move || {
        ui_handle.unwrap().hide().expect("Could not close Farbig Launcher");
    });

    ui.run()?;

    Ok(())
}