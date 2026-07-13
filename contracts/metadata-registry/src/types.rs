use soroban_sdk::{contracttype, Address, String, Vec};

/// Semantic category — mirrors what the AI classifier outputs
#[contracttype]
#[derive(Clone, PartialEq, Debug)]
pub enum TransactionCategory {
    Income,
    Expense,
    Transfer,
    Subscription,
    Savings,
    Payroll,
    Investment,
    Fee,
    Swap,
    Unknown,
}

/// Financial sentiment of the transaction
#[contracttype]
#[derive(Clone, PartialEq, Debug)]
pub enum TransactionSentiment {
    Positive,
    Neutral,
    Negative,
}

/// A tag is simply a short string label applied to a transaction.
/// Examples: "salary", "streaming", "Q2-2025", "recurring"
pub type Tag = String;

/// Full metadata record attached to one Stellar transaction hash
#[contracttype]
#[derive(Clone, Debug)]
pub struct Metadata {
    /// Stellar transaction hash (hex string)
    pub tx_hash: String,
    /// Wallet this metadata belongs to
    pub owner: Address,
    /// AI or manually assigned category
    pub category: TransactionCategory,
    /// Positive / Neutral / Negative
    pub sentiment: TransactionSentiment,
    /// Searchable tags e.g. ["food", "monthly", "recurring"]
    pub tags: Vec<Tag>,
    /// User-defined label e.g. "Netflix Payment"
    pub label: Option<String>,
    /// Free-form user notes
    pub notes: Option<String>,
    /// Human-readable counterparty name e.g. "Netflix", "Salary from Acme"
    pub counterparty_label: Option<String>,
    /// Whether this tx is part of a recurring pattern
    pub is_recurring: bool,
    /// ID from the recurring-registry contract (if linked)
    pub recurring_id: Option<u64>,
    /// AI classification confidence score 0–100
    pub ai_confidence: u32,
    /// Ledger when metadata was first written
    pub created_at_ledger: u64,
    /// Ledger when metadata was last updated
    pub updated_at_ledger: u64,
}
