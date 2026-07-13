use soroban_sdk::{contracttype, Address, String};

/// How often a recurring payment fires
#[contracttype]
#[derive(Clone, PartialEq, Debug)]
pub enum Frequency {
    Daily,
    Weekly,
    BiWeekly,
    Monthly,
    Quarterly,
    Annually,
    Custom,
}

/// Semantic category of the recurring payment
#[contracttype]
#[derive(Clone, PartialEq, Debug)]
pub enum SubscriptionType {
    Subscription, // Netflix, Spotify, SaaS
    Payroll,      // outgoing salary
    Income,       // incoming recurring (salary received, rent)
    Savings,      // auto-save transfers
    Bill,         // utilities, rent, insurance
    Investment,   // DCA, staking top-ups
    Transfer,     // wallet-to-wallet scheduled
    Other,
}

/// Lifecycle state of a subscription
#[contracttype]
#[derive(Clone, PartialEq, Debug)]
pub enum SubscriptionStatus {
    Active,
    Paused,
    Cancelled,
    Expired,
}

/// Core recurring payment record stored on-chain
#[contracttype]
#[derive(Clone, Debug)]
pub struct Subscription {
    /// Auto-incremented ID
    pub id: u64,
    /// Wallet that owns this subscription
    pub owner: Address,
    /// Human-readable merchant/counterparty name
    pub merchant: String,
    /// Optional on-chain address of the merchant
    pub merchant_address: Option<Address>,
    /// Payment frequency
    pub frequency: Frequency,
    /// Semantic category
    pub subscription_type: SubscriptionType,
    /// Asset code e.g. "USDC", "XLM"
    pub asset_code: String,
    /// Asset issuer address, or "native" for XLM
    pub asset_issuer: String,
    /// Payment amount in base units (stroops / smallest denomination)
    pub amount: i128,
    /// Current lifecycle status
    pub active: bool,
    pub status: SubscriptionStatus,
    /// Ledger sequence of next expected payment
    pub next_payment_ledger: u64,
    /// Ledger sequence of last confirmed payment
    pub last_payment_ledger: u64,
    /// Ledger sequence when this record was created
    pub created_at_ledger: u64,
    /// Whether the backend auto-detected this (vs manually added)
    pub auto_detected: bool,
    /// Optional user-defined label override
    pub custom_label: Option<String>,
}
