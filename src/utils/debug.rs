use std::{
    fmt::Debug,
    fs::{File, OpenOptions},
    io::Write,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use chrono::Local;
use serde::Deserialize;

use crate::Configuration;

/// The debug client for the database.
///
/// This is used to log debug information to a file.
#[derive(Debug, Clone)]
pub struct DebugClient {
    /// The path to the debug file.
    pub path: Option<PathBuf>,
    /// The file to write debug logs to.
    pub file: Option<Arc<Mutex<File>>>,
    /// The debug level for the database.
    pub level: DebugLevel,
    /// Whether or not to enable debug mode for the database.
    pub enabled: bool,
}

/// The debug level for the database.
#[derive(Debug, Clone, Deserialize)]
pub enum DebugLevel {
    Info,
    Warning,
    Error,
}

impl DebugClient {
    /// Create a new debug client.
    ///
    /// `enabled` - Whether or not to enable debug mode for the database.
    ///
    /// `level` - The debug level for the database.
    ///
    /// `config` - The configuration for the database. (Passed from the MooClient)
    pub fn new(enabled: bool, level: Option<DebugLevel>, config: Configuration) -> Self {
        let d_level = match level {
            Some(level) => level,
            None => DebugLevel::Info,
        };

        if !enabled {
            return Self {
                enabled: false,
                level: d_level,
                path: None,
                file: None,
            };
        }

        let db_dir_path = config.db_dir;
        let file_path = PathBuf::from(format!("{}/debug.log", db_dir_path));

        println!("Debug file path: {:?}", file_path);

        let file = match OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&file_path)
        {
            Ok(file) => file,
            Err(e) => {
                println!(
                    "Error opening debug file: {} ... attempting to create one.",
                    e
                );

                match File::create(&file_path) {
                    Ok(file) => file,
                    Err(e) => {
                        println!("Failed creating debug file: {}", e);
                        return Self {
                            enabled: true,
                            level: d_level,
                            path: None,
                            file: None,
                        };
                    }
                }
            }
        };

        Self {
            enabled: true,
            level: d_level,
            path: Some(file_path),
            file: Some(Arc::new(Mutex::new(file))),
        }
    }

    // todo - fix log function where logs don't overwrite old logs.
    /// Log a debug message to the debug file.
    /// 
    /// `debug` - The debug message or struct to log. This can be any data type that implements the Debug trait.
    /// 
    /// This function is internal and can't be used outside of the library.
    pub fn log<T>(&mut self, debug: T)
    where
        T: Debug,
    {
        if !self.enabled {
            return;
        }

        let current_time = Local::now();
        println!("{:?} - {:?}", current_time, debug);

        if let Some(file) = &self.file {
            let mut file = file.lock().unwrap();

            let _ = file.write_all(
                format!("[{}] {:?} - {:?}\n", current_time, self.level, debug).as_bytes(),
            );
        }
    }
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_debug() {
        let mut debug = DebugClient::new(true, None, Configuration::default());

        for i in 0..100 {
            debug.log(format!("Debug index #{} out of #{}", i, 100));
        }

        assert_eq!(debug.enabled, true);
    }
}
