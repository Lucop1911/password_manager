use std::path::PathBuf;

fn data_dir() -> PathBuf {
    let home_dir = dirs::home_dir().expect("Unable to find home directory");
    let app_dir = home_dir.join("p_manager");

    if !app_dir.exists() {
        let _ = std::fs::create_dir_all(&app_dir);
    }

    app_dir
}

pub fn db_path() -> PathBuf {
    data_dir().join("data.db")
}

pub fn keyfile_path() -> PathBuf {
    data_dir().join("keyfile.json")
}

pub fn legacy_path() -> PathBuf {
    data_dir().join("data.json")
}

pub fn legacy_backup_path() -> PathBuf {
    data_dir().join("data.json.migrated")
}
