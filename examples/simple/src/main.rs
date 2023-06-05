use moodb::MooClient;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
struct CryptoAccount {
    pub username: String,
    pub address: String,
    pub balance: f64,
}

const TABLE_NAME: &str = "crypto_accounts";

fn main() {
    // Create our database client
    let mut client: MooClient<CryptoAccount> = match MooClient::new(TABLE_NAME, None, None) {
        Ok(client) => client,
        Err(e) => panic!("Error creating client: {}", e.message),
    };

    client.reset_table().unwrap();

    // Get our table from the database
    let mut account_table = match client.get_table() {
        Ok(table) => table,
        Err(e) => panic!("Error getting table: {}", e.message),
    };

    // Insert some accounts into the table
    for i in 0..100 {
        let account = CryptoAccount {
            username: format!("user{}", i),
            address: format!("0x{}", i),
            balance: i as f64,
        };

        let key = format!("user{}", i);

        match account_table.insert(key.as_str(), account.clone()) {
            Ok(_) => println!("Inserted account: {}", account.username),
            Err(e) => println!("Error inserting account: {}", e.message),
        }
    }

    // Get an account from the table
    let account1 = match account_table.get("user1") {
        Ok(account) => account,
        Err(e) => panic!("Error getting account: {}", e.message),
    };

    println!("Account 1: {:?}", account1);

    // get account 50
    let account50 = match account_table.get("user50") {
        Ok(account) => account,
        Err(e) => panic!("Error getting account: {}", e.message),
    };

    println!("Account 50: {:?}", account50);

    // switch account 50 and 1 balances

    let u_account1 = CryptoAccount {
        username: account1.username,
        address: account1.address,
        balance: account50.balance,
    };

    let u_account50 = CryptoAccount {
        username: account50.username,
        address: account50.address,
        balance: account1.balance,
    };

    match account_table.update("user1", u_account1) {
        Ok(_) => println!("Updated account 1"),
        Err(e) => println!("Error updating account 1: {}", e.message),
    }

    match account_table.update("user50", u_account50) {
        Ok(_) => println!("Updated account 50"),
        Err(e) => println!("Error updating account 50: {}", e.message),
    }

    // get account 1
    let account1 = match account_table.get("user1") {
        Ok(account) => account,
        Err(e) => panic!("Error getting account: {}", e.message),
    };

    println!("Updated Account 1: {:?}", account1);

    // get account 50
    let account50 = match account_table.get("user50") {
        Ok(account) => account,
        Err(e) => panic!("Error getting account: {}", e.message),
    };

    println!("Updated Account 50: {:?}", account50);

    // delete account 50
    match account_table.delete("user50") {
        Ok(_) => println!("Deleted account 50"),
        Err(e) => println!("Error deleting account 50: {}", e.message),
    }

    // get account 50
    assert_eq!(account_table.get("user50").is_err(), true);
}
