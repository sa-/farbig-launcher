// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;
use std::fs;
use std::process::{Command, Stdio};
use std::time::Duration;

use slint::{Image, ModelRc, SharedString, VecModel};

slint::include_modules!();

fn declared_icon(resources: &std::path::Path, bundle_path: &std::path::Path) -> Option<std::path::PathBuf> {
    let output = Command::new("plutil")
        .args(["-extract", "CFBundleIconFile", "raw"])
        .arg(bundle_path.join("Contents/Info.plist"))
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let name = String::from_utf8(output.stdout).ok()?;
    let name = name.trim();
    (!name.is_empty()).then(|| resources.join(if name.ends_with(".icns") {
        name.to_owned()
    } else {
        format!("{name}.icns")
    }))
}

fn icon(bundle_path: &std::path::Path, index: usize) -> Image {
    let resources = bundle_path.join("Contents/Resources");
    let Ok(entries) = fs::read_dir(&resources) else {
        return Image::default();
    };
    let icon_path = declared_icon(&resources, bundle_path).filter(|path| path.is_file()).or_else(|| {
        entries.filter_map(Result::ok).map(|entry| entry.path()).find(|path| {
            path.extension().is_some_and(|extension| extension.eq_ignore_ascii_case("icns"))
        })
    });
    let Some(icon_path) = icon_path else {
        return Image::default();
    };

    let output_path = std::env::temp_dir().join(format!("farbig-launcher-{}-{index}.png", std::process::id()));
    let converted = Command::new("sips")
        .args(["--resampleHeightWidthMax", "192", "-s", "format", "png"])
        .arg(&icon_path)
        .arg("--out")
        .arg(&output_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success());

    let image = converted
        .then(|| Image::load_from_path(&output_path).ok())
        .flatten()
        .unwrap_or_default();
    let _ = fs::remove_file(output_path);
    image
}

fn apps() -> Result<Vec<App>, std::io::Error> {
    let mut bundles = fs::read_dir("/Applications")?
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            (path.extension().is_some_and(|extension| extension == "app")).then_some(path)
        })
        .enumerate()
        .filter_map(|(index, path)| {
            let name = path.file_stem()?.to_string_lossy().into_owned();
            let icon = icon(&path, index);
            let path = path.to_string_lossy().into_owned();

            Some(App {
                name: SharedString::from(name),
                path: SharedString::from(path),
                icon,
            })
        })
        .collect::<Vec<_>>();

    bundles.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(bundles)
}

fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;
    ui.set_apps(ModelRc::new(VecModel::from(apps()?)));

    let launch_handle = ui.as_weak();
    ui.on_launch_app(move |app_path| {
        if let Err(error) = Command::new("open").arg(app_path.as_str()).spawn() {
            eprintln!("Could not launch {app_path}: {error}");
        }
        let launch_handle = launch_handle.clone();
        slint::Timer::single_shot(Duration::from_secs(4), move || {
            let Some(ui) = launch_handle.upgrade() else {
                return;
            };
            ui.set_showing_launch_message(false);
        });
    });

    let ui_handle = ui.as_weak();
    ui.on_quit(move || {
        ui_handle.unwrap().hide().expect("Could not close Farbig Launcher");
    });

    ui.run()?;

    Ok(())
}