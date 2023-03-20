use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Return Type for common db actions
pub type MooResult<T> = Result<T, MooError>;

pub type MooRecords = Vec<Record>;

/// A record in the database.
///
/// This is a simple struct that contains an ID and a value.
///
/// Later this will be able to be customized by the user using there own struct that
/// implements the `Serialize` and `Deserialize` traits. It will contain and id and any
/// other data they want to store in a struct.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Record {
    /// The Key of the record and how data will be accessed.
    pub key: String,
    /// The value of the record.
    pub value: Value,
}

/// The storage type of the database.
///
/// This is an enum that contains the different storage types that the database can use.
///
/// The default storage type is `StorageType::Json`.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum StorageTypes {
    Memory,
    Json,
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
