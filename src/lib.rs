//! # MooDB
//!
//! MooDB is a simple, fast, and file persistence key-value database for Rust.
//!
//! ## Features
//!
//! - **Simple**: MooDB is simple to use and easy to learn.
//! - **Fast**: MooDB is fast and efficient using memory and disk API's.
//! - **File Persistence**: MooDB uses the filesystem to store data in a simple JSON format.
//! - **Key-Value**: MooDB is a key-value database.
//! - **Rust**: MooDB is written in Rust.
//! - **Thread Safe**: MooDB is thread safe by default.
//!
//! ## Design
//!
//! MooDB is designed to be used in a variety of applications. For example, MooDB can be used in a web server to store user data, or in a game to store player data.
//! Its file based so its a local database and can't be used over a network (yet).
//!
//! ## Usage
//! ```
//! use moodb::core::MooClient;
//!
//! struct Bank {
//!     balance: f32,
//!     ssn: String,
//! }
//!
//! fn main() {
//!     let mut db = MooClient::<String>::new("test", None, None).unwrap();
//!
//!     let mut user_accounts = db.get_table().unwrap();
//!
//!     user_accounts.insert("1", Bank {
//!        balance: 100.0,
//!        ssn: String::from("123-45-6789"),
//!     }).unwrap();
//!
//!     let user = user_accounts.get("1").unwrap();
//!
//!     println!("User: {:?}", user);
//!
//!     db.delete_table().unwrap();
//! }
//!
//! ```
//!
//! MooDB main functionality comes from using the built-in methods on a table for your database.
//!
//! These methods are:
//!
//! Client:
//!
//! - `new`: Creates a new client for the database.
//! - `get_table`: Gets a table from the database.
//! - `reset_table`: Resets a table in the database. (Keeps the db file.)
//! - `delete_table`: Deletes a table from the database. (Deletes the db file.)
//!
//! Table:
//!
//! - `insert`: Inserts a record into the table.
//! - `insert_many`: Inserts many records into the table.
//! - `get`: Gets a record from the table.
//! - `get_many`: Gets many records from the table.
//! - `get_all`: Gets all records from the table.
//! - `delete`: Deletes a record from the table.
//! - `delete_many`: Deletes many records from the table.
//! - `delete_all`: Deletes all records from the table.
//! - `update`: Updates a record in the table.
//! - `update_many`: Updates many records in the table.
//!
//! You can find more detailed information in the core module documentation.
//!

////////////////////////////////////////////////////////////////////////////////

use serde::{Deserialize, Serialize};
use utils::debug::DebugLevel;

pub mod core;
mod utils;

const FILE_EXTENSION: &str = "json";
const DEFAULT_DIR: &str = "db/moo";

#[derive(Debug, Clone)]
/// Configuration for the database.
pub struct Configuration {
    /// The directory to store the database file.
    pub db_dir: &'static str,
    /// Whether or not to enable debug mode for the database.
    pub debug_mode: bool,
    /// The debug level for the database.
    pub debug_level: Option<DebugLevel>,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            db_dir: DEFAULT_DIR,
            debug_mode: false,
            debug_level: Some(DebugLevel::Info),
        }
    }
}

/// Return Type for common db actions
pub type MooResult<T> = Result<T, MooError>;
pub type MooRecords<T> = Vec<MooRecord<T>>;

/// A record in the database.
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

/// The error struct for the database.
#[derive(Debug)]
pub struct MooError {
    pub code: MooErrorCodes,
    pub message: String,
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

mod tests {

    #[allow(unused_imports)]
    use crate::Configuration;
    #[allow(unused_imports)]
    use crate::{core::MooClient, utils::debug::DebugLevel};

    #[test]
    fn insert() {
        let mut db = MooClient::<String>::new(
            "test",
            None,
            Some(Configuration {
                db_dir: "db/moo",
                debug_mode: true,
                debug_level: Some(DebugLevel::Info),
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
