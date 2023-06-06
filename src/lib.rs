mod debug;

use debug::{DebugClient, DebugLevel};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::sync::{Arc, Mutex};
use std::{fs, path::PathBuf};

const FILE_EXTENSION: &str = "json";
const DEFAULT_DIR: Option<&str> = Some("db/moo");

#[derive(Debug, Clone)]
/// Configuration for the database.
pub struct Configuration {
    pub db_dir: Option<&'static str>,
    pub debug_mode: bool,
    pub debug_level: DebugLevel,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            db_dir: DEFAULT_DIR,
            debug_mode: false,
            debug_level: DebugLevel::Info,
        }
    }
}

/// Return Type for common db actions
pub type MooResult<T> = Result<T, MooError>;
pub type MooRecords<T> = Vec<MooRecord<T>>;

/// A record in the database.
///
/// This is a simple struct that contains an ID and a value.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct MooRecord<T>
where
    T: Serialize,
{
    /// The Key of the record and how data will be accessed.
    pub key: String,
    /// The value of the record.
    pub value: T,
}

/// The error types for the database.
#[derive(Debug)]
pub enum MooErrorCodes {
    NotFound,
    Debug,
    Warn,
    Error,
    Fatal,
}

/// The error struct for the database.
#[derive(Debug)]
pub struct MooError {
    pub code: MooErrorCodes,
    pub message: String,
}

/// The main database client.
///
/// This struct is used to create a new database instance
/// and perform actions on the database.
#[derive(Debug, Clone)]
pub struct MooClient<T>
where
    T: Clone + Serialize + DeserializeOwned,
{
    /// The director where the database is stored.
    pub path: PathBuf,

    pub table: MooTable<T>,

    pub config: Configuration,

    pub debugger: DebugClient,
}

impl<T> MooClient<T>
where
    T: Clone + Serialize + DeserializeOwned,
{
    /// Creates a new Moo database instance.
    ///
    /// The `name` of the table for this database instance is required.
    ///
    /// Pass the `path` to the directory where the database and its tables will be stored.
    /// If non is passed, the database will be stored in a default directory called `moodb` in the current working directory.
    ///
    /// Returns a `MooResult` with the result of the action.
    pub fn new(
        name: &str,
        dir: Option<&str>,
        config: Option<Configuration>,
    ) -> MooResult<MooClient<T>> {
        println!("MooDB Initializing...");

        let config = match config {
            Some(config) => config,
            None => Configuration::default(),
        };

        let config_clone = config.clone();

        let path = match dir {
            Some(dir) => PathBuf::from(dir),
            None => PathBuf::from(format!("./{}", DEFAULT_DIR.unwrap())),
        };

        if !path.exists() {
            match fs::create_dir_all(&path) {
                Ok(_) => {},
                Err(_) => {
                    return Err(MooError {
                        code: MooErrorCodes::Fatal,
                        message: "Failed to create database directory. Might be missing permissions to write the directory?".to_string()
                    })
                }
            }
        }

        let _debugger = DebugClient::new(false, DebugLevel::Info, config_clone);

        let table = match MooTable::new(name, &path, config.clone(), _debugger.clone()) {
            Ok(table) => table,
            Err(err) => {
                return Err(err);
            }
        };

        println!("MooDB Initialized.");

        Ok(Self {
            path,
            table,
            config,
            debugger: _debugger,
        })
    }

    /// Reset the table file and clear all records.
    pub fn reset_table(&mut self) -> MooResult<()> {
        self.debugger
            .log(format!("Resetting table: {}", self.table.name));

        self.table.records.clear();

        let mut file = match self.table.file.lock() {
            Ok(file) => file,
            Err(_) => {
                return Err(MooError {
                    code: MooErrorCodes::Fatal,
                    message: "Failed to lock table file.".to_string(),
                })
            }
        };

        match file.seek(SeekFrom::Start(0)) {
            Ok(_) => {}
            Err(_) => {
                return Err(MooError {
                    code: MooErrorCodes::Fatal,
                    message: "Failed to seek table file.".to_string(),
                })
            }
        }

        match file.write_all(&[]) {
            Ok(_) => {}
            Err(_) => {
                return Err(MooError {
                    code: MooErrorCodes::Fatal,
                    message: "Failed to write to table file.".to_string(),
                })
            }
        }

        match file.set_len(0) {
            Ok(_) => {}
            Err(_) => {
                return Err(MooError {
                    code: MooErrorCodes::Fatal,
                    message: "Failed to truncate table file.".to_string(),
                })
            }
        }

        match file.flush() {
            Ok(_) => {}
            Err(_) => {
                return Err(MooError {
                    code: MooErrorCodes::Fatal,
                    message: "Failed to flush table file.".to_string(),
                })
            }
        }

        Ok(())
    }

    /// Get a table from the database.
    ///
    /// Pass the `name` of the table to get.
    ///
    /// Returns a `MooResult` with the result of the action.
    pub fn get_table(&mut self) -> MooResult<MooTable<T>> {
        self.debugger
            .log(format!("Getting table: {}", self.table.name));

        Ok(self.table.clone())
    }

    /// Delete the table file itself.
    ///
    /// Returns a `MooResult` with the result true if the table was deleted, false if it was not or an error if something went wrong.
    pub fn delete_table(&mut self) -> MooResult<()> {
        self.debugger
            .log(format!("Deleting table: {}", self.table.name));

        match self.table.delete_self(&self.path) {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MooTable<T>
where
    T: Clone + Serialize + DeserializeOwned,
{
    pub name: String,
    pub file: Arc<Mutex<File>>,
    pub records: MooRecords<T>,
    pub config: Configuration,
    pub debugger: DebugClient,
}

impl<T> MooTable<T>
where
    T: Clone + Serialize + DeserializeOwned,
{
    /// Creates a new table for a database instance.
    ///
    /// The `name` of the table to create.
    ///
    /// The `path` to the directory where the table will be stored.
    ///
    /// This is an internal function and can't be used directly by the user.
    fn new(
        name: &str,
        path: &PathBuf,
        config: Configuration,
        debugger: DebugClient,
    ) -> MooResult<MooTable<T>> {
        let file_path = path.join(format!("{}.{}", name, FILE_EXTENSION));

        let mut file = match OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&file_path) {
            Ok(file) => file,
            Err(_) => {
                match File::create(&file_path) {
                    Ok(file) => file,
                    Err(_) => {
                        return Err(MooError {
                            code: MooErrorCodes::Fatal,
                            message: "Failed to create table file. Might be missing permissions to write the directory?".to_string()
                        })
                    }
                }
            }
        };

        let mut contents = Vec::new();

        match file.read_to_end(&mut contents) {
            Ok(_) => {}
            Err(_) => {
                return Err(MooError {
                    code: MooErrorCodes::Fatal,
                    message: "Failed to read table file.".to_string(),
                })
            }
        }

        let records: Vec<MooRecord<T>> = if contents.is_empty() {
            Vec::new()
        } else {
            let cloned_contents = contents.clone(); // Create a clone for deserialization
            match serde_json::from_slice(&cloned_contents) {
                Ok(records) => records,
                Err(_) => {
                    return Err(MooError {
                        code: MooErrorCodes::Error,
                        message: "Failed to parse table file.".to_string(),
                    })
                }
            }
        };

        Ok(Self {
            name: name.to_string(),
            file: Arc::new(Mutex::new(file)),
            records,
            config,
            debugger,
        })
    }

    /// Deletes this table from the database instance.
    ///
    /// This is an internal function and can't be used directly by the user.
    fn delete_self(&mut self, path: &PathBuf) -> MooResult<()> {
        self.records.clear();

        let file_path = path.join(format!("{}.{}", self.name, FILE_EXTENSION));

        match fs::remove_file(&file_path) {
            Ok(_) => Ok(()),
            Err(_) => Err(MooError {
                code: MooErrorCodes::Fatal,
                message: format!("Failed to delete table file: {}. Might be missing permissions to delete the file.", self.name),
            })
        }
    }

    /// Saves the table to disk after an action.
    ///
    /// This is an internal function and can't be used directly by the user.
    fn save(&self) -> MooResult<()> {
        let serialized_records = match serde_json::to_vec(&self.records) {
            Ok(serialized_records) => serialized_records,
            Err(_) => {
                return Err(MooError {
                    code: MooErrorCodes::Error,
                    message: "Failed to serialize table records.".to_string(),
                })
            }
        };

        let mut file = match self.file.lock() {
            Ok(file) => file,
            Err(_) => {
                return Err(MooError {
                    code: MooErrorCodes::Fatal,
                    message: "Failed to lock table file.".to_string(),
                })
            }
        };

        match file.seek(SeekFrom::Start(0)) {
            Ok(_) => {}
            Err(_) => {
                return Err(MooError {
                    code: MooErrorCodes::Fatal,
                    message: "Failed to seek table file.".to_string(),
                })
            }
        }

        match file.write_all(&serialized_records) {
            Ok(_) => {}
            Err(_) => {
                return Err(MooError {
                    code: MooErrorCodes::Fatal,
                    message: "Failed to write to table file.".to_string(),
                })
            }
        }

        match file.set_len(serialized_records.len() as u64) {
            Ok(_) => {}
            Err(_) => {
                return Err(MooError {
                    code: MooErrorCodes::Fatal,
                    message: "Failed to truncate table file.".to_string(),
                })
            }
        }

        match file.flush() {
            Ok(_) => {}
            Err(_) => {
                return Err(MooError {
                    code: MooErrorCodes::Fatal,
                    message: "Failed to flush table file.".to_string(),
                })
            }
        }

        Ok(())
    }

    /// Insert a new record into the table.
    ///
    /// The `key` of the record to insert.
    ///
    /// The `value` of the record to insert.
    pub fn insert(&mut self, key: &str, value: T) -> MooResult<()> {
        let exist = match self.get(key) {
            Ok(_) => true,
            Err(_) => false,
        };

        if exist {
            return Err(MooError {
                code: MooErrorCodes::Warn,
                message: format!("Record with key: {} already exists. Use the update method to change its value.", key),
            });
        }

        let record = MooRecord {
            key: key.to_string(),
            value,
        };

        self.records.push(record);

        match self.save() {
            Ok(_) => {}
            Err(err) => {
                return Err(err);
            }
        }

        self.debugger
            .log(format!("Inserted new record with key: {}", key));

        Ok(())
    }

    /// Get a record from the table.
    ///
    /// The `key` of the record to get.
    ///
    /// Returns a `MooResult` with the result of the action.
    pub fn get(&mut self, key: &str) -> MooResult<T> {
        for record in &self.records {
            if record.key == key {
                self.debugger.log(format!("Found record with key: {}", key));

                return Ok(record.value.clone());
            }
        }

        Err(MooError {
            code: MooErrorCodes::NotFound,
            message: format!("No record found with key: {}", key),
        })
    }

    /// Delete a record from the table.
    ///
    /// The `key` of the record to delete.
    ///
    /// Returns a `MooResult` with the result of the action.
    pub fn delete(&mut self, key: &str) -> MooResult<()> {
        let mut index = 0;

        for record in &self.records {
            if record.key == key {
                self.records.remove(index);
                self.save()?;

                self.debugger
                    .log(format!("Deleted record with key: {}", key));

                return Ok(());
            }

            index += 1;
        }

        Err(MooError {
            code: MooErrorCodes::NotFound,
            message: format!("No record found with key: {}", key),
        })
    }

    /// Update a record in the table.
    ///
    /// The `key` of the record to update.
    ///
    /// The `value` of the record to update.
    ///
    /// Returns a `MooResult` with the result of the action.
    pub fn update(&mut self, key: &str, value: T) -> MooResult<()> {
        let mut index = 0;

        for record in &self.records {
            if record.key == key {
                self.records[index].value = value;
                self.save()?;

                self.debugger
                    .log(format!("Updated record with key: {}", key));

                return Ok(());
            }

            index += 1;
        }

        Err(MooError {
            code: MooErrorCodes::NotFound,
            message: format!("No record found with key: {}", key),
        })
    }
}

mod tests {

    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn insert() {
        let mut db = MooClient::<String>::new(
            "test",
            None,
            Some(Configuration {
                db_dir: None,
                debug_mode: true,
                debug_level: DebugLevel::Info,
            }),
        )
        .unwrap();

        db.reset_table().unwrap();

        let mut people = db.get_table().unwrap();

        people.insert("1", "John".to_string()).unwrap();

        assert_eq!(people.records.len(), 1);

        people.insert("2", "Jane".to_string()).unwrap();

        assert_eq!(people.records.len(), 2);

        assert_eq!(people.get("1").unwrap(), "John".to_string());

        assert_eq!(people.delete("1").unwrap(), ());

        assert_eq!(people.records.len(), 1);

        // db.delete_table().unwrap();
    }
}
