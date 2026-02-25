#![cfg(test)]

//! Stress tests for arithmetic operations with very large i128 values in savings_goals
//!
//! These tests verify that the savings_goals contract handles extreme values correctly:
//! - Values near i128::MAX/2 to avoid overflow in additions
//! - Proper error handling for overflow conditions using checked_add/checked_sub
//! - No unexpected panics or wrap-around behavior
//!
//! ## Documented Limitations
//! - Maximum safe goal amount: i128::MAX/2 (to allow for safe addition operations)
//! - add_to_goal uses checked_add internally and will panic with "overflow" on overflow
//! - withdraw_from_goal uses checked_sub internally and will panic with "underflow" on underflow
//! - No explicit caps are imposed by the contract, but overflow/underflow will panic
//! - batch_add_to_goals has same limitations as add_to_goal for each contribution

use savings_goals::{ContributionItem, SavingsGoalContract, SavingsGoalContractClient};
use soroban_sdk::testutils::{Address as AddressTrait, Ledger, LedgerInfo};
use soroban_sdk::{Env, String, Vec};

fn set_time(env: &Env, timestamp: u64) {
    let proto = env.ledger().protocol_version();
    env.ledger().set(LedgerInfo {
        protocol_version: proto,
        sequence_number: 1,
        timestamp,
        network_id: [0; 32],
        base_reserve: 10,
        min_temp_entry_ttl: 1,
        min_persistent_entry_ttl: 1,
        max_entry_ttl: 100000,
    });
}

#[test]
fn test_create_goal_near_max_i128() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SavingsGoalContract);
    let client = SavingsGoalContractClient::new(&env, &contract_id);
    let owner = <soroban_sdk::Address as AddressTrait>::generate(&env);

    env.mock_all_auths();

    // Test with i128::MAX / 2 - a very large but safe value
    let large_target = i128::MAX / 2;

    let goal_id = client.create_goal(
        &owner,
        &String::from_str(&env, "Large Goal"),
        &large_target,
        &2000000,
    );

    let goal = client.get_goal(&goal_id).unwrap();
    assert_eq!(goal.target_amount, large_target);
    assert_eq!(goal.current_amount, 0);
}