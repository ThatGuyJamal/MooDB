#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;

use types::{MooError, StorageType, Record};

mod types;

pub struct MooDB {
    // database file path
    pub path: PathBuf,
    // database file
    pub file: File,
    // database storage type
    pub storage_option: Option<StorageType>,
}

impl MooDB {
    /// Creates a new MooDB instance.
    ///
    /// If `custom_file_name` is `None`, the database
    /// will be created in the current directory with the name `moodb.json`.
    pub fn new(custom_file_name: Option<String>, custom_storage_option: Option<StorageType>) -> Result<Self, MooError> {
        let path = match custom_file_name {
            Some(name) => PathBuf::from(name),
            None => PathBuf::from("moodb.json"),
        };

        let storage_option = match custom_storage_option {
            Some(option) => Some(option),
            None => None,
        };

        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(&path)
            .map_err(|e| MooError {
                code: types::MooErrorCodes::Error,
                message: format!("{}", e),
            })?;

        Ok(Self { path, file, storage_option })
    }

    /// Inserts a record into the database.
    ///
    /// If the record already exists, it will be overwritten.
    pub fn insert(&mut self, record: &Record) -> Result<(), MooError> {
        todo!()
    }

    /// Selects a record from the database.
    ///
    /// If the record does not exist, `None` will be returned.
    pub fn select(&mut self, id: u32) -> Result<Option<Record>, MooError> {
        todo!()
    }

    /// Reads the entire database file into memory.
    ///
    /// This is a helper function that is used by `insert` and `select`.
    fn read_all(&mut self) -> Result<Vec<u8>, types::MooError> {
        todo!()
    }
}
