# MooDB

A simple file based database using key-value pairs.

## Example

```rs
use moodb::MooClient;

struct Bank {
    balance: f64,
    age: u8,
}

fn main() {
    let mut db = MooClient::<String>::new("bank_accounts", None, None).unwrap();

    let accounts = db.get_table().unwrap();

    let bank_data = Bank {
        balance: 100.0,
        age: 20,
    };

    accounts.insert("John Doe", bank_data).unwrap(); // adds data to the db

    accounts.get("John Doe").unwrap(); // returns the data saved

    let updated_bank_data = Bank {
        balance: 200.0,
        age: 21,
    };

    accounts.update("John Doe", updated_bank_data).unwrap(); // updates the data saved

    accounts.delete("John Doe").unwrap(); // deletes the data saved
}

```

For more complex examples, see the [examples](./examples) directory.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
# ...
moodb = { git = "https://github.com/ThatGuyJamal/MooDB" }
```

## Todo

- `delete_many` method
- `delete_all` method
- `update_many` method
- `get_many` method
- `get_all` method
- `insert_many` method

- Add memory old mode (for faster reads/write) but no file persistance.
- Improve thread safety
- improve error messages and error handling
- Add to crates.io
