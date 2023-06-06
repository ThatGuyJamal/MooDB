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

#[derive(Debug, Clone)]
pub struct DebugClient {
    pub path: Option<PathBuf>,
    pub file: Option<Arc<Mutex<File>>>,
    pub level: DebugLevel,
    pub enabled: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub enum DebugLevel {
    Info,
    Warning,
    Error,
}

impl DebugClient {
    pub fn new(enabled: bool, level: DebugLevel, config: Configuration) -> Self {
        if !enabled {
            return Self {
                enabled: false,
                level: DebugLevel::Info,
                path: None,
                file: None,
            };
        }

        let db_dir_path = config.db_dir.expect("No db_dir");
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
                    "Error opening debug file: {} ... attempting to create one",
                    e
                );

                match File::create(&file_path) {
                    Ok(file) => file,
                    Err(e) => {
                        println!("Failed creating debug file: {}", e);
                        return Self {
                            enabled: true,
                            level: DebugLevel::Info,
                            path: None,
                            file: None,
                        };
                    }
                }
            }
        };

        Self {
            enabled: true,
            level,
            path: Some(file_path),
            file: Some(Arc::new(Mutex::new(file))),
        }
    }

    // todo - fix log function where logs don't overwrite old logs.
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
        let mut debug = DebugClient::new(true, DebugLevel::Info, Configuration::default());

        for i in 0..100 {
            debug.log(format!("Debug index #{} out of #{}", i, 100));
        }

        assert_eq!(debug.enabled, true);
    }
}
