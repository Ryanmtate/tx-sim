use std::collections::HashMap;
use std::io::{self, Write};
use std::path::PathBuf;

use csv::{Reader, Writer};
use rand::distributions::{Distribution, Uniform};
use rand::thread_rng;

use crate::*;

#[derive(Debug, Default)]
pub struct Accounting {
    accounts: HashMap<ClientId, Account>,
    transactions: HashMap<TxId, Transaction>,
}

impl Accounting {
    /// Wrapper method for creating a default Accounting struct;
    pub fn init() -> Self {
        Accounting::default()
    }

    /// This method is provided to manually lock the account;
    /// If an client account is locked after a chargeback, no transactions may be processed until it is
    /// unlocked.
    pub fn lock_account(&mut self, client: ClientId, is_locked: bool) -> () {
        let mut account = self
            .accounts
            .remove(&client)
            .unwrap_or_else(|| Account::new(client));

        // Update the locked status on the account;
        account.locked = is_locked;

        self.accounts.insert(client, account);
    }

    /// This is the main method for processing the transaction;
    /// NOTE: If the client does not already have an account, this transaction
    /// will also create an account for the client.
    pub fn process_transaction(&mut self, tx: Transaction) -> () {
        // Find or create a new account;
        let mut account = self
            .accounts
            .remove(&tx.client)
            .unwrap_or_else(|| Account::new(tx.client));

        // Only process the transaction if the account is unlocked;
        // NOTE: Another method will need to be used to unlock an account
        // after a charge back;
        if !account.locked {
            // Process the transaction and account based on transaction type;
            match tx.r#type {
                TxType::Deposit => {
                    self.process_deposit(&mut account, &tx);
                    // NOTE: Only insert the transaction is it a deposit or withdrawal;
                    // If it is part of dispute resolution, the tx id is the same as the deposit tx id;
                    self.transactions.insert(tx.tx, tx.clone());
                }
                TxType::Withdrawal => {
                    self.process_withdrawal(&mut account, &tx);
                    // NOTE: Only insert the transaction is it a deposit or withdrawal;
                    // If it is part of dispute resolution, the tx id is the same as the deposit tx id;
                    self.transactions.insert(tx.tx, tx.clone());
                }
                TxType::Dispute => self.process_dispute(&mut account, &tx),
                TxType::Resolve => self.process_resolve(&mut account, &tx),
                TxType::Chargeback => self.process_chargeback(&mut account, &tx),
                TxType::Unknown => unreachable!(),
            }
        }

        // update changes (if any) for account;
        self.accounts.insert(tx.client, account);
    }

    /// This method is provided as a helper method and is exposed for convience, but is intended to be consumed by
    /// `self.process_transaction`
    pub fn process_deposit(&mut self, account: &mut Account, tx: &Transaction) -> () {
        if let Some(amount) = tx.amount {
            // Credit the client's account
            account.total += amount;
            account.available += amount;
        }
    }
    /// This method is provided as a helper method and is exposed for convience, but is intended to be consumed by
    /// `self.process_transaction`
    pub fn process_withdrawal(&mut self, account: &mut Account, tx: &Transaction) -> () {
        if let Some(amount) = tx.amount {
            // Only if the account has sufficient funds will the account's values be updated;
            if account.available - amount >= 0. {
                // Debit the client's account;
                account.total -= amount;
                account.available -= amount;
            }
        }
    }
    /// This method is provided as a helper method and is exposed for convience, but is intended to be consumed by
    /// `self.process_transaction`
    pub fn process_dispute(&mut self, account: &mut Account, tx: &Transaction) -> () {
        // find the disputed transaction; If it does not exist, ignore.
        if let Some(transaction) = self.transactions.get(&tx.tx) {
            if let Some(amount) = transaction.amount {
                // Only if the account has sufficient available funds for dispute can they be held;
                // available funds cannot be negative;
                if account.available - amount >= 0. {
                    account.available -= amount;
                    account.held += amount;
                }
            }
        }
    }

    /// This method is provided as a helper method and is exposed for convience, but is intended to be consumed by
    /// `self.process_transaction`
    pub fn process_resolve(&mut self, account: &mut Account, tx: &Transaction) -> () {
        // find the transaction to resolve; If it does not exist, ignore.
        if let Some(transaction) = self.transactions.get(&tx.tx) {
            if let Some(amount) = transaction.amount {
                // Only if the account has previously disputed and held funds can the transaction be resolved;
                if account.held - amount >= 0. {
                    account.available += amount;
                    account.held -= amount;
                }
            }
        }
    }

    /// This method is provided as a helper method and is exposed for convience, but is intended to be consumed by
    /// `self.process_transaction`
    pub fn process_chargeback(&mut self, account: &mut Account, tx: &Transaction) -> () {
        // find the transaction to charge back; If it does not exist, ignore.
        if let Some(transaction) = self.transactions.get(&tx.tx) {
            if let Some(amount) = transaction.amount {
                // Only if the account has previously disputed and held funds can the transaction be charged back;
                if account.held - amount >= 0. {
                    // Decrease the total amount;
                    account.total -= amount;

                    // Decrease the funds held by the charge back amount;
                    account.held -= amount;

                    // Lock the account once they have had a charge back;
                    account.locked = true;
                }
            }
        }
    }

    /// Write accounts csv table to standard output
    pub fn write_accounts_csv_stdout(&mut self) -> Result<(), Error> {
        let mut wtr = Writer::from_writer(vec![]);

        for account in self.accounts.values_mut() {
            // Round balances before serialization;
            account.round_balances()?;

            wtr.serialize(account)?;
        }

        // Write Accounts csv to stdout;
        io::stdout().write_all(&wtr.into_inner()?)?;

        Ok(())
    }

    /// Used as a helper method to create dummy transactions;
    pub fn write_transactions_csv_file(
        transactions: Vec<Transaction>,
        file_path: PathBuf,
    ) -> Result<(), Error> {
        let mut wtr = Writer::from_path(file_path)?;

        for tx in transactions {
            wtr.serialize(&tx)?;
        }

        wtr.flush()?;

        Ok(())
    }

    /// Read the CSV transactions file and process each transaction;
    pub fn read_transactions_csv_file(&mut self, file_path: PathBuf) -> Result<(), Error> {
        let mut file = Reader::from_path(file_path)?;

        for row in file.deserialize::<Transaction>() {
            // Process CSV Row;
            let transaction = row?;

            // Process Transaction as it is being read;
            // Update client account from transaction;
            self.process_transaction(transaction);
        }

        Ok(())
    }

    /// Convenience method for getting an account stored in the private accounts HashMap
    pub fn get_account(&self, client: ClientId) -> Option<&Account> {
        self.accounts.get(&client)
    }

    /// Generate random transactions to be used for test data;
    /// Generated data may contain erroneous transactions on purpose;
    /// Use generated data to write test cases to enforce correctness;
    /// Data may also be used for performance testing;
    pub fn generate_dummy_transactions(
        num_transactions: u32,
        num_accounts: u16,
    ) -> Result<Vec<Transaction>, Error> {
        let mut rng = thread_rng();
        let mut transactions = Vec::new();

        let txs: Vec<u32> = Uniform::new_inclusive(1, num_transactions)
            .sample_iter(&mut rng)
            .take(num_transactions as usize)
            .collect();

        for tx in txs {
            let mut rng = thread_rng();
            let client: u16 = Uniform::new_inclusive(1, num_accounts)
                .sample_iter(&mut rng)
                .take(1)
                .sum();

            let tx_type: i32 = Uniform::new_inclusive(1, 5)
                .sample_iter(&mut rng)
                .take(1)
                .sum();

            let r#type = TxType::from(tx_type);

            let amount = match r#type {
                TxType::Deposit | TxType::Withdrawal => Some(
                    format!(
                        "{:.4}",
                        Uniform::new_inclusive(0.1, 500.)
                            .sample_iter(&mut rng)
                            .take(3)
                            .sum::<f64>()
                    )
                    .parse::<f64>()?,
                ),
                _ => None,
            };

            let transaction = Transaction {
                tx,
                client,
                r#type,
                amount,
            };

            transactions.push(transaction)
        }

        Ok(transactions)
    }
}
