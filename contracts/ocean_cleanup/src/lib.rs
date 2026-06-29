#![no_std]

//! # ocean_cleanup
//!
//! A Soroban smart contract that tracks ocean cleanup efforts on-chain.
//! Cleanup crews log cleanup events (location, kilograms collected, photo proof),
//! a designated captain verifies them, and verified events earn credit toward
//! a global leaderboard that ranks crews by total kilograms removed.

use soroban_sdk::{contract, contractimpl, contracttype, Address, BytesN, Env, Symbol};

/// Lifecycle status of a cleanup event.
///
/// - `Pending`  = logged by a crew, awaiting captain verification (0)
/// - `Verified` = a captain has signed off on the event (1)
/// - `Rewarded` = credit has been applied to the crew's leaderboard total (2)
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EventStatus {
    Pending = 0,
    Verified = 1,
    Rewarded = 2,
}

/// Persistent storage keys used by the contract.
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// Captain authorized to verify events (instance storage).
    Captain,
    /// Per-event record (persistent storage): id -> CleanupEvent.
    Event(Symbol),
    /// Per-crew accumulated kilograms (persistent storage): crew -> kg.
    CrewTotal(Address),
    /// Total kilograms cleaned across all crews (persistent storage).
    GlobalTotal,
}

/// On-chain record of a single cleanup event.
#[contracttype]
#[derive(Clone, Debug)]
pub struct CleanupEvent {
    pub event_id: Symbol,
    pub crew: Address,
    pub location_hash: BytesN<32>,
    pub kg_collected: u32,
    pub photo_hash: BytesN<32>,
    pub status: u32, // numeric form of EventStatus for cross-SDK compatibility
    pub verifier: Address,
    pub reward_kg: u32,
}

#[contract]
pub struct OceanCleanup;

#[contractimpl]
impl OceanCleanup {
    // ---------------------------------------------------------------------
    // Initialization
    // ---------------------------------------------------------------------

    /// Initialize the contract and set the crew captain (verifier) address.
    /// The captain is the only account that can mark events as verified.
    pub fn init(env: Env, captain: Address) {
        if env.storage().instance().has(&DataKey::Captain) {
            panic!("ocean_cleanup: contract already initialized");
        }
        env.storage().instance().set(&DataKey::Captain, &captain);
    }

    // ---------------------------------------------------------------------
    // Crew actions
    // ---------------------------------------------------------------------

    /// Log a new ocean cleanup event. The calling `crew` member must
    /// authorize the transaction. Stores a `Pending` event in persistent
    /// storage; captain verification is required before credit is applied.
    pub fn log_event(
        env: Env,
        crew: Address,
        event_id: Symbol,
        location_hash: BytesN<32>,
        kg_collected: u32,
        photo_hash: BytesN<32>,
    ) {
        crew.require_auth();

        if env.storage().persistent().has(&DataKey::Event(event_id.clone())) {
            panic!("ocean_cleanup: event already logged");
        }
        if kg_collected == 0 {
            panic!("ocean_cleanup: kg_collected must be greater than zero");
        }

        let event = CleanupEvent {
            event_id: event_id.clone(),
            crew: crew.clone(),
            location_hash,
            kg_collected,
            photo_hash,
            status: EventStatus::Pending as u32,
            verifier: crew.clone(), // placeholder; replaced on verify
            reward_kg: 0,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Event(event_id), &event);
    }

    // ---------------------------------------------------------------------
    // Captain actions
    // ---------------------------------------------------------------------

    /// Captain verifies a previously logged cleanup event. Transitions the
    /// event from `Pending` to `Verified`. Only the configured captain may
    /// call this and must authorize the transaction.
    pub fn verify_event(env: Env, captain: Address, event_id: Symbol) {
        captain.require_auth();

        let stored_captain: Address = env
            .storage()
            .instance()
            .get(&DataKey::Captain)
            .expect("ocean_cleanup: contract not initialized");
        if captain != stored_captain {
            panic!("ocean_cleanup: caller is not the designated captain");
        }

        let mut event: CleanupEvent = env
            .storage()
            .persistent()
            .get(&DataKey::Event(event_id.clone()))
            .expect("ocean_cleanup: event not found");

        if event.status != EventStatus::Pending as u32 {
            panic!("ocean_cleanup: event is not pending");
        }

        event.status = EventStatus::Verified as u32;
        event.verifier = captain;
        event.reward_kg = event.kg_collected;

        env.storage()
            .persistent()
            .set(&DataKey::Event(event_id), &event);
    }

    /// Issue a reward for a verified event. Credits the crew's running total
    /// and the global leaderboard with the event's `kg_collected`. Anyone
    /// may call this once the event is `Verified` — useful for an automated
    /// payout bot or an admin trigger.
    pub fn reward(env: Env, event_id: Symbol) {
        let mut event: CleanupEvent = env
            .storage()
            .persistent()
            .get(&DataKey::Event(event_id.clone()))
            .expect("ocean_cleanup: event not found");

        if event.status != EventStatus::Verified as u32 {
            panic!("ocean_cleanup: event must be verified before reward");
        }

        // Move the event to the Rewarded state.
        event.status = EventStatus::Rewarded as u32;
        env.storage()
            .persistent()
            .set(&DataKey::Event(event_id), &event);

        // Credit the crew's leaderboard total.
        let crew_key = DataKey::CrewTotal(event.crew.clone());
        let current: u32 = env
            .storage()
            .persistent()
            .get(&crew_key)
            .unwrap_or(0u32);
        env.storage()
            .persistent()
            .set(&crew_key, &(current + event.kg_collected));

        // Credit the global total.
        let global: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::GlobalTotal)
            .unwrap_or(0u32);
        env.storage()
            .persistent()
            .set(&DataKey::GlobalTotal, &(global + event.kg_collected));
    }

    // ---------------------------------------------------------------------
    // Views
    // ---------------------------------------------------------------------

    /// Return the running total kilograms credited to `crew` on the leaderboard.
    pub fn get_crew_total(env: Env, crew: Address) -> u32 {
        env.storage()
            .persistent()
            .get(&DataKey::CrewTotal(crew))
            .unwrap_or(0u32)
    }

    /// Return the current numeric status of an event:
    /// 0 = Pending, 1 = Verified, 2 = Rewarded.
    pub fn get_event_status(env: Env, event_id: Symbol) -> u32 {
        let event: CleanupEvent = env
            .storage()
            .persistent()
            .get(&DataKey::Event(event_id))
            .expect("ocean_cleanup: event not found");
        event.status
    }

    /// Return whether the given event has been verified by the captain.
    pub fn is_verified(env: Env, event_id: Symbol) -> bool {
        match env.storage().persistent().get::<DataKey, CleanupEvent>(
            &DataKey::Event(event_id),
        ) {
            Some(e) => e.status >= EventStatus::Verified as u32,
            None => false,
        }
    }

    /// Return the global total kilograms cleaned across all crews.
    pub fn get_global_total(env: Env) -> u32 {
        env.storage()
            .persistent()
            .get(&DataKey::GlobalTotal)
            .unwrap_or(0u32)
    }

    /// Return the full `CleanupEvent` record (or panic if it does not exist).
    pub fn get_event(env: Env, event_id: Symbol) -> CleanupEvent {
        env.storage()
            .persistent()
            .get(&DataKey::Event(event_id))
            .expect("ocean_cleanup: event not found")
    }
}
