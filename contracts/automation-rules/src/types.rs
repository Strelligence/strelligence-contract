use soroban_sdk::{contracttype, Address, String};

/// Type of automation rule
#[contracttype]
#[derive(Clone, PartialEq, Debug)]
pub enum RuleType {
    /// Move a % of income to a savings wallet
    AutoSave,
    /// Keep wallet balance within a range
    AutoSweep,
    /// Distribute salary to multiple recipients on schedule
    Payroll,
    /// Alert when a spending category exceeds a limit
    Budget,
    /// Generic notification trigger
    Alert,
}

/// What condition triggers this rule
#[contracttype]
#[derive(Clone, PartialEq, Debug)]
pub enum RuleTrigger {
    OnIncomingPayment,
    OnOutgoingPayment,
    OnSchedule,
    OnBalanceAbove,
    OnBalanceBelow,
    OnCategorySpend,
}

/// Rule lifecycle state
#[contracttype]
#[derive(Clone, PartialEq, Debug)]
pub enum RuleStatus {
    Active,
    Paused,
    Deleted,
}

/// A single programmable automation rule stored on-chain.
///
/// The contract is the source-of-truth registry.
/// The BACKEND reads rules and executes them — the contract does NOT execute.
/// `trigger_params` and `action_params` are JSON strings the backend parses.
#[contracttype]
#[derive(Clone, Debug)]
pub struct Rule {
    pub id: u64,
    pub owner: Address,
    pub rule_type: RuleType,
    pub trigger: RuleTrigger,
    pub status: RuleStatus,
    /// Human-readable description
    pub label: String,
    /// JSON string of trigger parameters e.g. '{"min_amount":1000000}'
    pub trigger_params: String,
    /// JSON string of action parameters e.g. '{"percentage":10,"dest":"G..."}'
    pub action_params: String,
    pub created_at_ledger: u64,
    pub last_executed_ledger: u64,
    pub execution_count: u64,
}