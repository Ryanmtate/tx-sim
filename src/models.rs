use serde::{Deserialize, Serialize};

use super::Error;

/// Valid u16 client ID; Client IDs exceeding u16::MAX will be considered invalid;
pub type ClientId = u16;

/// Globally unique, unordered u32 transaction ID; Transaction IDs exceeding u32::MAX will be considered invalid;
pub type TxId = u32;

/// The possible transactions types representing a transaction.
/// This structure provides the match arm expressions for determining
/// transactions processing logic.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum TxType {
    #[serde(rename = "deposit")]
    /// A deposit is a credit to the client’s asset account, meaning it should **increase the available and
    /// total funds** of the client account
    Deposit,
    #[serde(rename = "withdrawal")]
    /// A withdraw is a debit to the client’s asset account, meaning it should **decrease the available and
    /// total funds** of the client account. If a client does not have sufficient available funds the withdrawal
    // should fail and the total amount of funds should not change
    Withdrawal,
    #[serde(rename = "dispute")]
    /// A dispute represents a client’s claim that a transaction was erroneous and should be reverse. The
    /// transaction shouldn’t be reversed yet but the associated funds should be held. This means that the
    /// clients **available funds should decrease** by the amount disputed, their **held funds should increase**
    /// by the amount disputed, while their *total funds should remain the same*.
    Dispute,
    #[serde(rename = "resolve")]
    /// A resolve represents a resolution to a dispute, releasing the associated held funds. Funds that were
    /// previously disputed are no longer disputed. This means that the clients **held funds should decrease**
    /// by the amount no longer disputed, their **available funds should increase** by the amount no longer
    /// disputed, and **their total funds should remain the same**.
    Resolve,
    #[serde(rename = "chargeback")]
    /// A chargeback is the final state of a dispute and represents the client reversing a transaction. Funds
    /// that were held have now been withdrawn. This means that the clients **held funds and total funds
    /// should decrease by the amount** previously disputed. **If a chargeback occurs the client’s account
    /// should be immediately frozen**.
    Chargeback,
    #[serde(rename = "unknown")]
    /// an unknown transaction;
    Unknown,
}

impl From<i32> for TxType {
    fn from(num: i32) -> Self {
        match num {
            1 => TxType::Deposit,
            2 => TxType::Withdrawal,
            3 => TxType::Dispute,
            4 => TxType::Resolve,
            5 => TxType::Chargeback,
            _ => TxType::Unknown,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
/// Structure representing the transaction details provided in the input for processing.
pub struct Transaction {
    #[serde(rename = "type")]
    /// Transaction Type
    pub r#type: TxType,
    /// Globally unique, unordered u16 client ID
    #[serde(rename = "client")]
    pub client: ClientId,
    /// Globally unique, unordered u32 transaction ID
    #[serde(rename = "tx")]
    pub tx: TxId,
    /// Transaction amount, represented to four decimal places of precision
    #[serde(rename = "amount")]
    pub amount: Option<f64>,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
/// Structure representing the account details provided in the output for processing.
pub struct Account {
    /// Valid u16 client ID; Client IDs exceeding u16::MAX will be considered invalid.
    #[serde(rename = "client")]
    pub client: ClientId,
    /// The total funds that are available for trading, staking, withdrawal, etc.
    /// This should be equal to the total - held amounts
    #[serde(rename = "available")]
    pub available: f64,
    /// The total funds that are held for dispute. This should be equal to total - available amounts
    #[serde(rename = "held")]
    pub held: f64,
    /// The total funds that are available or held. This should be equal to available + held
    #[serde(rename = "total")]
    pub total: f64,
    /// Whether the account is locked. An account is locked if a charge back occurs
    #[serde(rename = "locked")]
    pub locked: bool,
}

impl Account {
    pub fn new(client: ClientId) -> Self {
        Account {
            client,
            ..Default::default()
        }
    }

    /// Helper method for rounding account balances to four decimal places;
    /// NOTE: This method would be better suited as an implemented Trait,
    /// reusable for other models.
    pub fn round_balances(&mut self) -> Result<(), Error> {
        self.total = format!("{:.4}", self.total).parse::<f64>()?;
        self.held = format!("{:.4}", self.held).parse::<f64>()?;
        self.available = format!("{:.4}", self.available).parse::<f64>()?;
        Ok(())
    }
}
