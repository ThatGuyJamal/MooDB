#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

mod types;

use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;

use serde_json::{json, Value};
use types::{MooError, MooErrorCodes, MooRecords, MooResult, Record, StorageTypes};

/// The main database client.
///
/// This struct is used to create a new database instance
/// and perform actions on the database.
pub struct MooClient {
    /// database file stored in memory
    pub path: PathBuf,
    /// database file stored in memory
    pub file: File,
    /// database storage type (json, ram, etc.)
    pub storage_type: StorageTypes,
    /// database records
    pub records: MooRecords,
}

impl MooClient {
    /// Creates a new MooDB instance.
    ///
    /// If `custom_file_name` is `None`, the database
    /// will be created in the current directory with the name `moo.json`.
    pub fn new(
        custom_file_name: Option<String>,
        custom_storage_type: Option<StorageTypes>,
    ) -> MooResult<Self> {
        let path = match custom_file_name {
            Some(name) => PathBuf::from(name),
            None => PathBuf::from("moo.json"),
        };

        let storage_type = match custom_storage_type {
            Some(option) => option,
            None => StorageTypes::Json,
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
    pub fn insert(&mut self, record: &Record) -> MooResult<bool> {
        let key = &record.key;
        let value = record.value.clone();

        // Check if the record already exists
        let exists = self.exists(Some(&key)).unwrap();

        // If it does, update it instead
        if exists {
            match self.update(&key, value) {
                Ok(_) => return Ok(true),
                Err(e) => {
                    return Err(MooError {
                        code: MooErrorCodes::Fatal,
                        message: e.message,
                    })
                }
            }
        }

        // Write the record to the database file
        self.write_to_file(record).unwrap();

        Ok(true)
    }

    /// Selects a record from the database.
    ///>
    /// If the record does not exist, `None` will be returned.
    pub fn select(&mut self, key: &String) -> MooResult<Option<Record>> {
        let records = match self.find_all_records() {
            Ok(records) => records,
            Err(e) => {
                return Err(MooError {
                    code: MooErrorCodes::Fatal,
                    message: e.message,
                })
            }
        };

        let exist = match self.exists(Some(&key)) {
            Ok(exist) => exist,
            Err(e) => {
                return Err(MooError {
                    code: MooErrorCodes::Fatal,
                    message: e.message,
                })
            }
        };

        if !exist {
            return Ok(None);
        }

        for record in records {
            if record.key == *key {
                return Ok(Some(record));
            }
        }

        return Ok(None)
    }

    /// Deletes a record from the database.
    pub fn delete(&mut self, key: String) -> MooResult<bool> {
        todo!()
    }

    /// Updates a record in the database.
    pub fn update(&mut self, key: &String, value: Value) -> MooResult<bool> {
        let mut records = match self.find_all_records() {
            Ok(records) => records,
            Err(e) => {
                return Err(MooError {
                    code: MooErrorCodes::Fatal,
                    message: e.message,
                })
            }
        };

        let mut found = false;

        for record in records.iter_mut() {
            let v = record.value.clone();
            if record.key == *key {
                record.value = v;
                found = true;
            }
        }

        let raw = &Record {
            key: key.clone(),
            value,
        };

        if !found {
            return match self.insert(&raw) {
                Ok(_) => Ok(true),
                Err(_) => Ok(false),
            };
        } else {
            return match self.write_to_file(&raw) {
                Ok(_) => Ok(true),
                Err(_) => Ok(false),
            };
        }
    }

    /// Returns the number of records in the database.
    pub fn count(&mut self) -> MooResult<usize> {
        todo!()
    }

    /// Checks if a record exists in the database.
    pub fn exists(&mut self, key: Option<&String>) -> MooResult<bool> {
        if let Some(key) = key {
            let records = match self.find_all_records() {
                Ok(records) => records,
                Err(e) => {
                    return Err(MooError {
                        code: MooErrorCodes::Fatal,
                        message: e.message,
                    })
                }
            };

            for record in records {
                if record.key == *key {
                    return Ok(true);
                }
            }

            return Ok(false);
        } else {
            return Ok(false);
        }
    }

    /// Writes a record to the database file.
    fn write_to_file(&mut self, data: &Record) -> MooResult<()> {
        let mut file = self.create_or_read_file().unwrap();

        // Read the current records
        let current_records = match self.find_all_records() {
            Ok(records) => records,
            Err(e) => {
                return Err(MooError {
                    code: MooErrorCodes::Fatal,
                    message: e.message,
                })
            }
        };

        let mut records = current_records.clone();

        // Add the new record
        records.push(data.clone());

        let json = serde_json::to_string(&records).unwrap();

        // Write the new records to the database file
        return match file.write_all(json.as_bytes()) {
            Ok(_) => Ok(()),
            Err(e) => {
                return Err(MooError {
                    code: MooErrorCodes::Fatal,
                    message: e.to_string(),
                })
            }
        };
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
    fn find_all_records(&mut self) -> MooResult<MooRecords> {
        return match self.storage_type {
            StorageTypes::Json => self.find_all_json_records(),
            StorageTypes::Memory => self.find_all_memory_records(),
        };
    }

    fn find_all_json_records(&mut self) -> MooResult<MooRecords> {
        self.records
            .iter()
            .map(|record| Ok(record.clone()))
            .collect()
    }

    fn find_all_memory_records(&mut self) -> MooResult<MooRecords> {
        Ok(self.records.clone())
    }

    /// Gets all the current keys in the database and returns them as a `Vec<String>`.
    fn find_all_keys(&mut self) -> MooResult<Vec<String>> {
        let keys = self.records.iter().map(|record| Ok(record.key.clone()));
        return keys.collect();
    }

    /// Converts a `Record` to a `String` based on the database's storage type.
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
    fn string_to_record(s: String) -> MooResult<Record> {
        return match serde_json::from_str(&s) {
            Ok(record) => Ok(record),
            Err(e) => Err(MooError {
                code: MooErrorCodes::Error,
                message: e.to_string(),
            }),
        };
    }

    /// Gets all the current records in the database and returns them as a `Vec<String>`.
    fn find_all_record_strings(&mut self) -> MooResult<Vec<String>> {
        let record_strings = self.records.iter().map(|record| {
            let string = serde_json::to_string(record).unwrap();
            Ok(string)
        });
        return record_strings.collect();
    }

    /// Converts a `Vec<String>` to a `Vec<Record>`.
    fn from_record_strings_to_records(record_strings: Vec<String>) -> MooResult<MooRecords> {
        let records = record_strings
            .iter()
            .map(|record_string| {
                let record: Record = serde_json::from_str(record_string).unwrap();
                Ok(record)
            })
            .collect();
        return records;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_record_vectors() {
        let mut db = MooClient::new(None, None).unwrap();

        let data = vec!["a", "b", "c"];

        let raw = Record {
            key: "test".to_string(),
            value: data.into(),
        };

        let json = db.record_to_string(&raw).unwrap();

        let record = MooClient::string_to_record(json).unwrap();

        assert_eq!(record.key, "test");
        assert_eq!(record.value, json!(["a", "b", "c"]));
        assert_eq!(record.value.as_array().unwrap().len(), 3);
    }

    #[test]
    fn test_insert_and_select() {
        let mut db = MooClient::new(None, None).unwrap();

        let data = json!({
            "name": "John Doe",
            "age": 30,
            "is_active": true,
        });

        let raw = Record {
            key: "test".to_string(),
            value: data,
        };

        db.insert(&raw).unwrap();

        let data2 = json!({
            "name": "John Doe 2",
            "age": 60,
            "is_active": false,
        });

        let raw2 = Record {
            key: "test2".to_string(),
            value: data2,
        };

        db.insert(&raw2).unwrap();

        let record = match db.select(&"test".to_string()) {
            Ok(record) => record,
            Err(e) => {
                panic!("{}", e.message);
            }
        };

        println!("{:?}", record);  

        if let Some (record) = record {
            assert_eq!(record.key, "test");
            assert_eq!(record.value["name"], "John Doe");
            assert_eq!(record.value["age"], 30);
            assert_eq!(record.value["is_active"], true);
        }
    }
}
