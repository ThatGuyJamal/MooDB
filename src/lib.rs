#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

mod types;

use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;

use serde_json::json;
use types::{MooError, MooRecord, MooResult, Record, StorageType, MooErrorCodes};

pub struct MooDB {
    // database file path
    pub path: PathBuf,
    // database file
    pub file: File,
    // database storage type
    pub storage_type: StorageType,
    // database records
    pub records: MooRecord,
}

impl MooDB {
    /// Creates a new MooDB instance.
    ///
    /// If `custom_file_name` is `None`, the database
    /// will be created in the current directory with the name `moo.json`.
    pub fn new(
        custom_file_name: Option<String>,
        custom_storage_type: Option<StorageType>,
    ) -> MooResult<Self> {
        let path = match custom_file_name {
            Some(name) => PathBuf::from(name),
            None => PathBuf::from("moo.json"),
        };

        let storage_type = match custom_storage_type {
            Some(option) => option,
            None => StorageType::Json,
        };

        let file = match OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)
        {
            Ok(file) => file,
            Err(e) => {
                return Err(MooError {
                    code: MooErrorCodes::Error,
                    message: e.to_string(),
                })
            }
        };

        Ok(Self {
            path,
            file,
            storage_type,
            records: vec![],
        })
    }

    /// Inserts a record into the database.
    ///
    /// If the record already exists, it will be overwritten.
    pub fn insert(&mut self, record: &Record) -> MooResult<()> {
        let json = json!(record);
        //
        todo!()
    }

    /// Selects a record from the database.
    ///
    /// If the record does not exist, `None` will be returned.
    pub fn select(&mut self, id: u32) -> MooResult<Record> {
        todo!()
    }

    /// Deletes a record from the database.
    pub fn delete(&mut self, id: u32) -> MooResult<bool> {
        todo!()
    }

    /// Updates a record in the database.
    pub fn update(&mut self, id: String, record: &Record) -> MooResult<()> {
        todo!()
    }

    /// Returns the number of records in the database.
    pub fn count(&mut self) -> MooResult<usize> {
        todo!()
    }

    /// Checks if a record exists in the database.
    pub fn exists(&mut self, id: u32) -> MooResult<bool> {
        todo!()
    }

    /// Creates or reads the database file.
    fn create_or_read_file(&mut self) -> MooResult<File> {
        let file = match OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&self.path)
        {
            Ok(file) => file,
            Err(e) => {
                return Err(MooError {
                    code: MooErrorCodes::Error,
                    message: e.to_string(),
                })
            }
        };

        Ok(file)
    }

    /// Reads the entire database file into memory.
    ///
    /// This is a helper function that is used by `insert` and `select`.
    fn find_all_records(&mut self) -> MooResult<MooRecord> {
        todo!()
    }

    /// Converts a `Record` to a `String` based on the database's storage type.
    /// 
    /// This is a helper function that is used by `insert` and `select`.
    fn record_to_string(&mut self, record: &Record) -> MooResult<String> {
        return match serde_json::to_string(record) {
            Ok(json) => Ok(json),
            Err(e) => Err(MooError {
                code: MooErrorCodes::Error,
                message: e.to_string(),
            }),
        };
    }

    /// Converts a `String` to a `Record` based on the database's storage type.
    /// 
    /// This is a helper function that is used by `insert` and `select`.
    fn string_to_record(s: String) -> MooResult<Record> {
        return match serde_json::from_str(&s) {
            Ok(record) => Ok(record),
            Err(e) => Err(MooError {
                code: MooErrorCodes::Error,
                message: e.to_string(),
            }),
        };
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use super::*;

    #[test]
    fn test_record_vectors() {
        let mut db = MooDB::new(None, None).unwrap();
        
        let data = vec!["a", "b", "c"];
        
        let raw = Record {
            key: "test".to_string(),
            value: data.into()
        };

        let json = db.record_to_string(&raw).unwrap();

        let record = MooDB::string_to_record(json).unwrap();

        assert_eq!(record.key, "test");
        assert_eq!(record.value, json!(["a", "b", "c"]));
        assert_eq!(record.value.as_array().unwrap().len(), 3);
    }
}
