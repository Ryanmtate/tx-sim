use std::path::PathBuf;

use crate::{Accounting, Error, Transaction, TxType};

#[test]
fn test_account_deposit() -> Result<(), Error> {
    let mut accounting = Accounting::init();

    let client = 1;
    let deposit_amount = 100.0;

    accounting.process_transaction(Transaction {
        client,
        tx: 1,
        r#type: TxType::Deposit,
        amount: Some(deposit_amount),
    });

    // Assert the account is created when making a deposit;
    assert_eq!(accounting.get_account(client).is_some(), true);

    // Assert both the account total and available balance are set;
    assert_eq!(
        accounting.get_account(client).map(|a| a.total),
        Some(deposit_amount)
    );
    assert_eq!(
        accounting.get_account(client).map(|a| a.available),
        Some(deposit_amount)
    );

    Ok(())
}

#[test]
fn test_account_withdrawal() -> Result<(), Error> {
    let mut accounting = Accounting::init();

    let client = 1;
    let deposit_amount = 100.0;
    let withdrawal_amount = 40.0;

    accounting.process_transaction(Transaction {
        client,
        tx: 1,
        r#type: TxType::Deposit,
        amount: Some(deposit_amount),
    });

    accounting.process_transaction(Transaction {
        client,
        tx: 2,
        r#type: TxType::Withdrawal,
        amount: Some(withdrawal_amount),
    });

    // Ensure account total is reduced by amount withdrawn
    assert_eq!(
        accounting.get_account(client).map(|a| a.total),
        Some(deposit_amount - withdrawal_amount)
    );

    assert_eq!(
        accounting.get_account(client).map(|a| a.available),
        Some(deposit_amount - withdrawal_amount)
    );

    Ok(())
}

#[test]
fn test_account_dispute() -> Result<(), Error> {
    let mut accounting = Accounting::init();

    let client = 1;
    let deposit_amount = 100.0;

    accounting.process_transaction(Transaction {
        client,
        tx: 1,
        r#type: TxType::Deposit,
        amount: Some(deposit_amount),
    });

    accounting.process_transaction(Transaction {
        client,
        tx: 1,
        r#type: TxType::Dispute,
        amount: None,
    });

    assert_eq!(
        accounting.get_account(client).map(|a| a.total),
        Some(deposit_amount)
    );

    assert_eq!(
        accounting.get_account(client).map(|a| a.available),
        Some(0.0)
    );

    assert_eq!(
        accounting.get_account(client).map(|a| a.held),
        Some(deposit_amount)
    );

    Ok(())
}

#[test]
fn test_account_resolution() -> Result<(), Error> {
    let mut accounting = Accounting::init();

    let client = 1;
    let deposit_amount = 100.0;

    accounting.process_transaction(Transaction {
        client,
        tx: 1,
        r#type: TxType::Deposit,
        amount: Some(deposit_amount),
    });

    accounting.process_transaction(Transaction {
        client,
        tx: 1,
        r#type: TxType::Dispute,
        amount: None,
    });

    // Ensure account total available is reduced by disputed tx amount
    assert_eq!(
        accounting.get_account(client).map(|a| a.total),
        Some(deposit_amount)
    );

    assert_eq!(
        accounting.get_account(client).map(|a| a.available),
        Some(0.0)
    );

    assert_eq!(
        accounting.get_account(client).map(|a| a.held),
        Some(deposit_amount)
    );

    accounting.process_transaction(Transaction {
        client,
        tx: 1,
        r#type: TxType::Resolve,
        amount: None,
    });

    // Ensure account total available is available after resolution;
    assert_eq!(
        accounting.get_account(client).map(|a| a.total),
        Some(deposit_amount)
    );

    assert_eq!(
        accounting.get_account(client).map(|a| a.available),
        Some(deposit_amount)
    );

    assert_eq!(accounting.get_account(client).map(|a| a.held), Some(0.0));

    Ok(())
}

#[test]
fn test_account_chargeback() -> Result<(), Error> {
    let mut accounting = Accounting::init();

    let client = 1;
    let deposit_amount = 100.0;

    accounting.process_transaction(Transaction {
        client,
        tx: 1,
        r#type: TxType::Deposit,
        amount: Some(deposit_amount),
    });

    accounting.process_transaction(Transaction {
        client,
        tx: 1,
        r#type: TxType::Dispute,
        amount: None,
    });

    // Ensure account total available is reduced by disputed tx amount
    assert_eq!(
        accounting.get_account(client).map(|a| a.total),
        Some(deposit_amount)
    );

    assert_eq!(
        accounting.get_account(client).map(|a| a.available),
        Some(0.0)
    );

    assert_eq!(
        accounting.get_account(client).map(|a| a.held),
        Some(deposit_amount)
    );

    // Test transaction charge back

    accounting.process_transaction(Transaction {
        client,
        tx: 1,
        r#type: TxType::Chargeback,
        amount: None,
    });

    // Ensure account total available is available after resolution;
    assert_eq!(accounting.get_account(client).map(|a| a.total), Some(0.0));

    // Ensure available funds are reduced by funds withdrawn after charge back;
    assert_eq!(
        accounting.get_account(client).map(|a| a.available),
        Some(0.0)
    );

    // Ensure amount held after charge back is 0.0
    assert_eq!(accounting.get_account(client).map(|a| a.held), Some(0.0));

    // Ensure account is locked;
    assert_eq!(accounting.get_account(client).map(|a| a.locked), Some(true));

    // Test manual account unlock
    accounting.lock_account(client, false);

    // Ensure account is unlocked after manual unlocking;
    assert_eq!(
        accounting.get_account(client).map(|a| a.locked),
        Some(false)
    );

    Ok(())
}

#[test]
fn test_generate_transactions() -> Result<(), Error> {
    let num_transactions = 1000;
    let num_accounts = 10;

    let mut accounting = Accounting::init();

    for tx in Accounting::generate_dummy_transactions(num_transactions, num_accounts)? {
        accounting.process_transaction(tx);
    }

    accounting.write_accounts_csv_stdout()?;

    Ok(())
}

#[test]
fn test_write_csv_dummy_transactions() -> Result<(), Error> {
    let num_transactions = 1000;
    let num_accounts = 10;

    let file_path = PathBuf::from("transactions.csv");

    // Generate Dummy Transactions;
    let transactions = Accounting::generate_dummy_transactions(num_transactions, num_accounts)?;

    // Write transactions to CSV file;
    Accounting::write_transactions_csv_file(transactions, file_path)?;

    Ok(())
}
