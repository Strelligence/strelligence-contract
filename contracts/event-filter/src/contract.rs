use soroban_sdk::{contract, contractimpl, Address, Bytes, Env, Symbol, Vec};

use crate::{
    errors::ContractError,
    events,
    storage::{DataKey, INITIAL_VERSION, MAX_PAGE_SIZE, STORAGE_TTL},
    types::{EventAggregation, EventFilter, EventRecord, PaginatedEvents},
};

#[contract]
pub struct EventFilterContract;

#[contractimpl]
impl EventFilterContract {
    // ─────────────────────────────────────────────────────────────────────────
    // INITIALIZATION
    // ─────────────────────────────────────────────────────────────────────────

    /// Initialize the event filter contract.
    pub fn initialize(env: Env, admin: Address) -> Result<(), ContractError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(ContractError::AlreadyInitialized);
        }

        let placeholder = Bytes::from_array(&env, &[0u8; 32]);
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::NextEventId, &0u64);
        env.storage()
            .instance()
            .set(&DataKey::Version, &INITIAL_VERSION);
        env.storage()
            .instance()
            .set(&DataKey::WasmHash, &placeholder);

        env.storage().instance().extend_ttl(0, 6_312_000);

        Ok(())
    }

    // ─────────────────────────────────────────────────────────────────────────
    // EVENT INDEXING
    // ─────────────────────────────────────────────────────────────────────────

    /// Record an event for indexing. Only admin can call this.
    pub fn record_event(
        env: Env,
        caller: Address,
        contract_address: Address,
        topic: Symbol,
        data: Bytes,
    ) -> Result<u64, ContractError> {
        caller.require_auth();

        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(ContractError::NotInitialized)?;

        if caller != admin {
            return Err(ContractError::NotAdmin);
        }

        let id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::NextEventId)
            .unwrap_or(0u64)
            + 1;

        let ledger = env.ledger().sequence() as u64;

        let event = EventRecord {
            id,
            contract_address: contract_address.clone(),
            topic: topic.clone(),
            data,
            ledger,
            timestamp: ledger,
        };

        env.storage()
            .instance()
            .set(&DataKey::EventRecord(id), &event);
        env.storage().instance().set(&DataKey::NextEventId, &id);

        env.storage().instance().extend_ttl(0, 6_312_000);

        events::event_indexed(&env, id);

        Ok(id)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // EVENT QUERYING
    // ─────────────────────────────────────────────────────────────────────────

    /// Query events with filtering and pagination.
    pub fn query_events(
        env: Env,
        filter: EventFilter,
        cursor: Option<u64>,
        limit: u32,
    ) -> Result<PaginatedEvents, ContractError> {
        let page_limit = if limit > MAX_PAGE_SIZE {
            MAX_PAGE_SIZE
        } else {
            limit
        };

        let total_events: u64 = env
            .storage()
            .instance()
            .get(&DataKey::NextEventId)
            .unwrap_or(0u64);

        let start_id = cursor.unwrap_or(1);
        let mut results: Vec<EventRecord> = Vec::new(&env);
        let mut count = 0u64;
        let mut last_id = start_id;

        let mut id = start_id;
        while id <= total_events && count < page_limit as u64 {
            if let Some(event) = env
                .storage()
                .instance()
                .get::<_, EventRecord>(&DataKey::EventRecord(id))
            {
                let mut matches = true;

                if let Some(ref topic) = filter.topic {
                    if event.topic != *topic {
                        matches = false;
                    }
                }

                if let Some(from_ledger) = filter.from_ledger {
                    if event.ledger < from_ledger {
                        matches = false;
                    }
                }

                if let Some(to_ledger) = filter.to_ledger {
                    if event.ledger > to_ledger {
                        matches = false;
                    }
                }

                if let Some(ref addr) = filter.contract_address {
                    if event.contract_address != *addr {
                        matches = false;
                    }
                }

                if matches {
                    results.push_back(event);
                    count += 1;
                }
            }

            last_id = id;
            id += 1;
        }

        let next_cursor = if last_id < total_events {
            Some(last_id + 1)
        } else {
            None
        };

        let paginated = PaginatedEvents {
            events: results,
            next_cursor,
            total_count: total_events,
        };

        Ok(paginated)
    }

    /// Get events by topic.
    pub fn get_events_by_topic(env: Env, topic: Symbol, limit: u32) -> Vec<EventRecord> {
        let page_limit = if limit > MAX_PAGE_SIZE {
            MAX_PAGE_SIZE
        } else {
            limit
        };

        let total_events: u64 = env
            .storage()
            .instance()
            .get(&DataKey::NextEventId)
            .unwrap_or(0u64);

        let mut results: Vec<EventRecord> = Vec::new(&env);
        let mut count = 0u64;
        let mut id = 1u64;

        while id <= total_events && count < page_limit as u64 {
            if let Some(event) = env
                .storage()
                .instance()
                .get::<_, EventRecord>(&DataKey::EventRecord(id))
            {
                if event.topic == topic {
                    results.push_back(event);
                    count += 1;
                }
            }
            id += 1;
        }

        results
    }

    /// Get events in a ledger range.
    pub fn get_events_by_ledger_range(
        env: Env,
        from_ledger: u64,
        to_ledger: u64,
        limit: u32,
    ) -> Vec<EventRecord> {
        let page_limit = if limit > MAX_PAGE_SIZE {
            MAX_PAGE_SIZE
        } else {
            limit
        };

        let total_events: u64 = env
            .storage()
            .instance()
            .get(&DataKey::NextEventId)
            .unwrap_or(0u64);

        let mut results: Vec<EventRecord> = Vec::new(&env);
        let mut count = 0u64;
        let mut id = 1u64;

        while id <= total_events && count < page_limit as u64 {
            if let Some(event) = env
                .storage()
                .instance()
                .get::<_, EventRecord>(&DataKey::EventRecord(id))
            {
                if event.ledger >= from_ledger && event.ledger <= to_ledger {
                    results.push_back(event);
                    count += 1;
                }
            }
            id += 1;
        }

        results
    }

    /// Get aggregated event counts by topic.
    pub fn get_event_aggregation(
        env: Env,
        from_ledger: Option<u64>,
        to_ledger: Option<u64>,
    ) -> Vec<EventAggregation> {
        let total_events: u64 = env
            .storage()
            .instance()
            .get(&DataKey::NextEventId)
            .unwrap_or(0u64);

        let mut aggregations: soroban_sdk::Map<Symbol, EventAggregation> =
            soroban_sdk::Map::new(&env);

        let mut id = 1u64;
        while id <= total_events {
            if let Some(event) = env
                .storage()
                .instance()
                .get::<_, EventRecord>(&DataKey::EventRecord(id))
            {
                let mut in_range = true;

                if let Some(from) = from_ledger {
                    if event.ledger < from {
                        in_range = false;
                    }
                }

                if let Some(to) = to_ledger {
                    if event.ledger > to {
                        in_range = false;
                    }
                }

                if in_range {
                    if let Some(mut agg) = aggregations.get(event.topic.clone()) {
                        agg.count += 1;
                        if event.ledger < agg.first_ledger {
                            agg.first_ledger = event.ledger;
                        }
                        if event.ledger > agg.last_ledger {
                            agg.last_ledger = event.ledger;
                        }
                        aggregations.set(event.topic.clone(), agg);
                    } else {
                        let agg = EventAggregation {
                            topic: event.topic.clone(),
                            count: 1,
                            first_ledger: event.ledger,
                            last_ledger: event.ledger,
                        };
                        aggregations.set(event.topic.clone(), agg);
                    }
                }
            }
            id += 1;
        }

        let mut results: Vec<EventAggregation> = Vec::new(&env);
        for (_, agg) in aggregations.iter() {
            results.push_back(agg);
        }

        results
    }

    /// Get a single event by ID.
    pub fn get_event(env: Env, event_id: u64) -> Option<EventRecord> {
        env.storage()
            .instance()
            .get(&DataKey::EventRecord(event_id))
    }

    /// Get total event count.
    pub fn total_events(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::NextEventId)
            .unwrap_or(0u64)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // UPGRADE FUNCTIONS
    // ─────────────────────────────────────────────────────────────────────────

    /// Upgrade the contract to a new WASM hash. Only the admin can call this.
    pub fn upgrade(env: Env, caller: Address, new_wasm_hash: Bytes) -> Result<(), ContractError> {
        caller.require_auth();

        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(ContractError::NotInitialized)?;

        if caller != admin {
            return Err(ContractError::NotAdmin);
        }

        let current_hash: Bytes = env
            .storage()
            .instance()
            .get(&DataKey::WasmHash)
            .ok_or(ContractError::NotInitialized)?;

        if current_hash == new_wasm_hash {
            return Err(ContractError::SameWasmHash);
        }

        let version: u32 = env
            .storage()
            .instance()
            .get(&DataKey::Version)
            .unwrap_or(INITIAL_VERSION);
        let new_version = version + 1;

        env.storage()
            .instance()
            .set(&DataKey::WasmHash, &new_wasm_hash);
        env.storage()
            .instance()
            .set(&DataKey::Version, &new_version);

        env.storage().instance().extend_ttl(0, 6_312_000);

        events::contract_upgraded(&env, &admin, new_version);

        Ok(())
    }

    // ─────────────────────────────────────────────────────────────────────────
    // READ FUNCTIONS
    // ─────────────────────────────────────────────────────────────────────────

    /// Get the current admin address.
    pub fn get_admin(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::Admin)
    }

    /// Get the current contract version.
    pub fn get_version(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::Version)
            .unwrap_or(INITIAL_VERSION)
    }

    /// Get the current WASM hash stored on-chain.
    pub fn get_wasm_hash(env: Env) -> Option<Bytes> {
        env.storage().instance().get(&DataKey::WasmHash)
    }
}
