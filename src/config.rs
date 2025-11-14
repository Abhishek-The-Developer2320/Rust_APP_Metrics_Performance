use dotenvy::dotenv;
use std::env;
use std::path::PathBuf;

/// Get SQLite database URL (absolute path)
pub fn database_url() -> String {
    // Load environment variables from .env
    dotenv().ok();

    // Get DATABASE_URL from .env or default to task.db
    let db_path = env::var("DATABASE_URL").unwrap_or_else(|_| "task.db".to_string());

    // Convert to absolute path
    let mut path = PathBuf::from(&db_path);
    if !path.is_absolute() {
        path = std::env::current_dir().unwrap().join(path);
    }

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("Failed to create database folder");
    }

    // Ensure the file exists (try to create it). This makes sure we fail early with a clear error
    // if the directory is not writable.
    if !path.exists() {
        match std::fs::OpenOptions::new().create(true).write(true).open(&path) {
            Ok(_) => (),
            Err(e) => panic!("Failed to create database file {}: {}", path.display(), e),
        }
    }

    // SQLite connection string. On Windows absolute paths should be provided as URI with forward
    // slashes (e.g. sqlite:///C:/path/to/db). Convert backslashes to forward slashes when needed.
    let path_str = path.to_str().expect("Invalid database path");
    if path.is_absolute() {
        // normalize to forward slashes
        let norm = path_str.replace('\\', "/");
        format!("sqlite:///{}", norm)
    } else {
        format!("sqlite://{}", path_str)
    }
}
