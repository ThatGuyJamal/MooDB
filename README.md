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