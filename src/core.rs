use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::sync::{Arc, Mutex};
use std::{fs, path::PathBuf};

use crate::utils::debug::DebugClient;
use crate::{
    Configuration, MooError, MooErrorCodes, MooRecord, MooRecords, MooResult, DEFAULT_DIR,
    FILE_EXTENSION,
};

/// The main database client.
///
/// This struct is used to create a new database instance
/// and perform actions on the database.
#[derive(Debug, Clone)]
pub struct MooClient<T>
where
    T: Clone + Serialize + DeserializeOwned,
{
    /// The path to the directory where the database and its tables are stored.
    pub path: PathBuf,

    /// The table for this database instance.
    pub table: MooTable<T>,

    /// The configuration for this database instance.
    pub config: Configuration,

    /// The debugger for this database instance.
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
            None => PathBuf::from(format!("./{}", DEFAULT_DIR)),
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

        let _debugger = DebugClient::new(config.debug_mode, None, config_clone);

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

/// The database table containing records.
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
            .log(format!("Insert new record with key: {}", key));

        Ok(())
    }

    pub fn insert_many(&mut self, data: MooRecords<T>) -> MooResult<()> {
        if data.is_empty() {
            return Err(MooError {
                code: MooErrorCodes::Warn,
                message: "No records to insert.".to_string(),
            });
        }

        for record in &data {
            let exist = match self.get(&record.key) {
                Ok(_) => true,
                Err(_) => false,
            };

            if exist {
                return Err(MooError {
                    code: MooErrorCodes::Warn,
                    message: format!("Record with key: {} already exists. Use the update method to change its value.", record.key),
                });
            }
        }

        for record in &data {
            self.records.push(record.clone());
            self.debugger
                .log(format!("Insert new record with key: {}", record.key));
        }

        match self.save() {
            Ok(_) => {}
            Err(err) => {
                return Err(err);
            }
        }

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

    /// Get multiple records from the table at once.
    ///
    /// The `keys` of the records to get.
    ///
    /// Returns a `MooResult` with the result of the action.
    pub fn get_many(&mut self, keys: Vec<&str>) -> MooResult<MooRecords<T>> {
        let mut records = Vec::new();

        for record in &self.records {
            if keys.contains(&record.key.as_str()) {
                records.push(record.clone());
                self.debugger
                    .log(format!("Found record with key: {}", record.key));
            }
        }

        if records.is_empty() {
            return Err(MooError {
                code: MooErrorCodes::NotFound,
                message: format!("No records found with keys: {:?}", keys),
            });
        }

        Ok(records)
    }

    /// Get all the records from the table.
    ///
    /// This should be extremely fast as the records are already loaded into memory.
    /// You can also simply access the `records` field directly, however this method is provided for convenience
    /// and it supports error handling and logging out of the box. Also keep in mind data should never be modified directly
    /// using this property as it can cause data corruption with the file persistence.
    ///
    /// Returns a `MooResult` with the result of the action.
    pub fn get_all(&mut self) -> MooResult<MooRecords<T>> {
        if self.records.is_empty() {
            return Err(MooError {
                code: MooErrorCodes::NotFound,
                message: "No records found in the table.".to_string(),
            });
        }

        self.debugger
            .log(format!("Found {} records", self.records.len()));

        Ok(self.records.clone())
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

    /// Update multiple records in the table at once.
    ///
    /// The `update` vector containing the records to update.
    ///
    /// Returns a `MooResult` with the result of the action.
    pub fn update_many(&mut self, update: MooRecords<T>) -> MooResult<()> {
        if update.is_empty() {
            return Err(MooError {
                code: MooErrorCodes::Warn,
                message: "No records to update.".to_string(),
            });
        }

        for record in &mut self.records {
            for update_record in &update {
                if record.key == update_record.key {
                    record.value = update_record.value.clone();
                    self.debugger
                        .log(format!("Updated record with key: {}", record.key));
                }
            }
        }

        match self.save() {
            Ok(_) => Ok(()),
            Err(err) => {
                return Err(err);
            }
        }
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

    /// Deletes multiple records from the table at once.
    ///
    /// The `keys` of the records to delete.
    ///
    /// Returns a `MooResult` with the result of the action.
    pub fn delete_many(&mut self, keys: Vec<&str>) -> MooResult<()> {
        self.debugger
            .log(format!("Deleting records with keys: {:?}", keys));

        self.records
            .retain(|record| !keys.contains(&record.key.as_str()));

        match self.save() {
            Ok(_) => Ok(()),
            Err(err) => {
                return Err(err);
            }
        }
    }

    /// Deletes all the records from the table.
    ///
    /// Returns a `MooResult` with the result of the action.
    pub fn delete_all(&mut self) -> MooResult<()> {
        self.debugger.log(format!("Deleting all records"));

        self.records.clear();

        match self.save() {
            Ok(_) => Ok(()),
            Err(err) => {
                return Err(err);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::MooClient;
    use crate::{Configuration, MooRecord};

    #[test]
    fn test_delete_many() {
        let mut db = MooClient::<String>::new(
            "test_delete_many",
            None,
            Some(Configuration {
                db_dir: "db/moo",
                debug_mode: true,
                debug_level: None,
            }),
        )
        .unwrap();

        db.reset_table().unwrap();

        let mut people = db.get_table().unwrap();

        for i in 0..50 {
            let data = format!("Example Person {}", i);

            people.insert(&i.to_string(), data).unwrap();
        }

        assert_eq!(people.records.len(), 50);

        people.delete_many(vec!["1", "2", "3"]).unwrap();

        assert_eq!(people.records.len(), 47);

        let u = vec![
            MooRecord {
                key: "4".to_string(),
                value: "Example Person 4 updated".to_string(),
            },
            MooRecord {
                key: "5".to_string(),
                value: "Example Person 5 updated".to_string(),
            },
            MooRecord {
                key: "6".to_string(),
                value: "Example Person 6 updated".to_string(),
            },
        ];

        people.update_many(u).unwrap();

        assert_eq!(people.get("4").unwrap(), "Example Person 4 updated");
    }
}
