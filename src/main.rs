use std::env;
use std::path::PathBuf;

use tx_sim::*;

fn main() -> Result<(), Error> {
    if let Some(file_path) = env::args().skip(1).collect::<Vec<String>>().first() {
        let mut accounting = Accounting::init();

        // Read the incoming transactions file;
        accounting.read_transactions_csv_file(PathBuf::from(file_path))?;

        // Write the accounts statements after processing transactions;
        accounting.write_accounts_csv_stdout()?;
    }

    Ok(())
}
