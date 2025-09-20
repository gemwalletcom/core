use crate::Settings;
use std::path::PathBuf;

pub fn get_test_settings() -> Settings {
    find_settings_file()
        .and_then(|path| Settings::new_setting_path(path).ok())
        .expect("Failed to load Settings.yaml for tests. Make sure Settings.yaml exists in the project root.")
}

fn find_settings_file() -> Option<PathBuf> {
    let mut current_dir = std::env::current_dir().ok()?;

    loop {
        let settings_path = current_dir.join("Settings.yaml");
        if settings_path.exists() {
            return Some(settings_path);
        }

        if !current_dir.pop() {
            break;
        }
    }

    None
}