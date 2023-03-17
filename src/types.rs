use serde::{Deserialize, Serialize};

/// A record in the database. 
/// 
/// This is a simple struct that contains an ID and a value.
/// 
/// Later this will be able to be customized by the user using there own struct that
/// implements the `Serialize` and `Deserialize` traits. It will contain and id and any 
/// other data they want to store in a struct.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Record {
    id: u32,
    value: String,
}

pub enum StorageType {
    Memory,
    Json
}

#[derive(Debug)]
pub enum MooErrorCodes {
    Debug,
    Warn,
    Error,
    Fatal
}

#[derive(Debug)]
pub struct MooError {
    pub code: MooErrorCodes, 
    pub message: String
}