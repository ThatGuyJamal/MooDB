# MooDB

A simple file based database using key-value pairs.

## Example

```rs
use moodb::MooClient;

struct Bank {
    balance: f64,
    age: u8,
}

let mut db = MooClient::<String>::new("bank_accounts", None, None).unwrap();

let account = db.get_table().unwrap();

let bank_data = Bank {
    balance: 100.0,
    age: 20,
};

account.insert("John Doe", bank_data).unwrap(); // adds data to the db

account.get("John Doe").unwrap(); // returns the data saved

let updated_bank_data = Bank {
    balance: 200.0,
    age: 21,
};

account.update("John Doe", updated_bank_data).unwrap(); // updates the data saved

account.delete("John Doe").unwrap(); // deletes the data saved

```

For more complex examples, see the [examples](./examples) directory.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
# ...
moodb = { git = "https://github.com/ThatGuyJamal/MooDB" }
```

Coming soon to `crates.io`

## Todo 

- `delete_many` method
- `delete_all` method
- `update_many` method
- `get_many` method
- `get_all` method
- `insert_many` method

- Improve thread safety
- Implement debug logs config option
- improve error messages and error handling