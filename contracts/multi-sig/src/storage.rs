use soroban_sdk::{contracttype, Address};

#[contracttype]
pub enum DataKey {
    Admin,
    Signers,
    Threshold,
    Proposal(u64),
    NextProposalId,
    ProposalSigners(u64),
    SignerProposalVote(Address, u64),
    Version,
    WasmHash,
}

pub const STORAGE_TTL: u32 = 3_110_400;
pub const INITIAL_VERSION: u32 = 1;
