use serde::{Deserialize, Serialize};
use utils::debug::DebugLevel;

mod utils;
pub mod core;

const FILE_EXTENSION: &str = "json";
const DEFAULT_DIR: &str = "db/moo";

#[derive(Debug, Clone)]
/// Configuration for the database.
pub struct Configuration {
    pub db_dir: &'static str,
    pub debug_mode: bool,
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
