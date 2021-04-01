//! Transaction Simulator (tx_sim)
//!
//! # Example
//! The main binary can be executed using the following command:
//!
//! `cargo run -- transactions.csv > accounts.csv`
//!
//! The default binary program reads the file from command line arguments and writes account details to stdout.
//!
//! ```no_run
//! use std::path::PathBuf;
//! use std::env;
//! use tx_sim::{Accounting, Error};
//!
//!
//! fn main() -> Result<(), Error> {
//!     if let Some(file_path) = env::args().skip(1).collect::<Vec<String>>().first() {
//!         let mut accounting = Accounting::init();
//!
//!         // Read the incoming transactions file;
//!         accounting.read_transactions_csv_file(PathBuf::from(file_path))?;
//!
//!         // Write the accounts statements after processing transactions;
//!         accounting.write_accounts_csv_stdout()?;
//!     }
//!
//!     Ok(())
//! }
//!
//! ```
//!
//! Using the library:
//!
//! ```no_run
//! use tx_sim::{Accounting, Transaction, TxType, Error};
//!
//! let mut accounting = Accounting::init();
//! let client = 1;
//! let deposit_amount = 100.0;
//!
//! // Process a transaction for an account programmatically;
//! accounting.process_transaction(Transaction {
//!     client,
//!     tx: 1,
//!     r#type: TxType::Deposit,
//!     amount: Some(deposit_amount),
//! });
//!
//! // Assert the account is created when making a deposit;
//! assert_eq!(accounting.get_account(client).is_some(), true);
//!
//! // Assert the account total balance is equal to the deposit amount;
//! assert_eq!(accounting.get_account(client).map(|c| c.total), Some(deposit_amount));
//!
//! ```
//! # Errors & Trouble Shooting
//!
//! If the program fails to parse the CSV file, check to ensure there are no leading empty spaces in the client, tx or amount values.
//! This will cause the process to exit with an CsvError.
//!

mod accounting;
mod error;
mod models;

#[cfg(test)]
mod test;

pub use accounting::*;
pub use error::*;
pub use models::*;
