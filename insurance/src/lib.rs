#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Env, Map, String, Vec};

#[derive(Clone)]
#[contracttype]
pub struct InsurancePolicy {
    pub id: u32,
    pub name: String,
    pub coverage_type: String, // "health", "emergency", etc.
    pub monthly_premium: i128,
    pub coverage_amount: i128,
    pub active: bool,
    pub next_payment_date: u64, // Unix timestamp
}

#[contract]
pub struct Insurance;

#[contractimpl]
impl Insurance {
    /// Create a new insurance policy
    ///
    /// # Arguments
    /// * `name` - Name of the policy
    /// * `coverage_type` - Type of coverage (e.g., "health", "emergency")
    /// * `monthly_premium` - Monthly premium amount
    /// * `coverage_amount` - Total coverage amount
    ///
    /// # Returns
    /// The ID of the created policy
    pub fn create_policy(
        env: Env,
        name: String,
        coverage_type: String,
        monthly_premium: i128,
        coverage_amount: i128,
    ) -> u32 {
        // Validate input amounts
        if monthly_premium <= 0 {
            panic!("Monthly premium must be positive");
        }
        if coverage_amount <= 0 {
            panic!("Coverage amount must be positive");
        }

        let mut policies: Map<u32, InsurancePolicy> = env
            .storage()
            .instance()
            .get(&symbol_short!("POLICIES"))
            .unwrap_or_else(|| Map::new(&env));

        let next_id = env
            .storage()
            .instance()
            .get(&symbol_short!("NEXT_ID"))
            .unwrap_or(0u32)
            + 1;

        // Set next payment date to 30 days from now
        let next_payment_date = env.ledger().timestamp() + (30 * 86400);

        let policy = InsurancePolicy {
            id: next_id,
            name: name.clone(),
            coverage_type: coverage_type.clone(),
            monthly_premium,
            coverage_amount,
            active: true,
            next_payment_date,
        };

        policies.set(next_id, policy);
        env.storage()
            .instance()
            .set(&symbol_short!("POLICIES"), &policies);
        env.storage()
            .instance()
            .set(&symbol_short!("NEXT_ID"), &next_id);

        next_id
    }

    /// Pay monthly premium for a policy
    ///
    /// # Arguments
    /// * `policy_id` - ID of the policy
    ///
    /// # Returns
    /// True if payment was successful, false otherwise
    pub fn pay_premium(env: Env, policy_id: u32) -> bool {
        let mut policies: Map<u32, InsurancePolicy> = env
            .storage()
            .instance()
            .get(&symbol_short!("POLICIES"))
            .unwrap_or_else(|| Map::new(&env));

        if let Some(mut policy) = policies.get(policy_id) {
            if !policy.active {
                return false; // Policy is not active
            }

            // Update next payment date to 30 days from now
            policy.next_payment_date = env.ledger().timestamp() + (30 * 86400);

            policies.set(policy_id, policy);
            env.storage()
                .instance()
                .set(&symbol_short!("POLICIES"), &policies);
            true
        } else {
            false
        }
    }

    /// Get a policy by ID
    ///
    /// # Arguments
    /// * `policy_id` - ID of the policy
    ///
    /// # Returns
    /// InsurancePolicy struct or None if not found
    pub fn get_policy(env: Env, policy_id: u32) -> Option<InsurancePolicy> {
        let policies: Map<u32, InsurancePolicy> = env
            .storage()
            .instance()
            .get(&symbol_short!("POLICIES"))
            .unwrap_or_else(|| Map::new(&env));

        policies.get(policy_id)
    }

    /// Get all active policies
    ///
    /// # Returns
    /// Vec of active InsurancePolicy structs
    pub fn get_active_policies(env: Env) -> Vec<InsurancePolicy> {
        let policies: Map<u32, InsurancePolicy> = env
            .storage()
            .instance()
            .get(&symbol_short!("POLICIES"))
            .unwrap_or_else(|| Map::new(&env));

        let mut result = Vec::new(&env);
        let max_id = env
            .storage()
            .instance()
            .get(&symbol_short!("NEXT_ID"))
            .unwrap_or(0u32);

        for i in 1..=max_id {
            if let Some(policy) = policies.get(i) {
                if policy.active {
                    result.push_back(policy);
                }
            }
        }
        result
    }

    /// Get total monthly premium for all active policies
    ///
    /// # Returns
    /// Total monthly premium amount
    pub fn get_total_monthly_premium(env: Env) -> i128 {
        let active = Self::get_active_policies(env);
        let mut total = 0i128;
        for policy in active.iter() {
            total += policy.monthly_premium;
        }
        total
    }

    /// Deactivate a policy
    ///
    /// # Arguments
    /// * `policy_id` - ID of the policy
    ///
    /// # Returns
    /// True if deactivation was successful
    pub fn deactivate_policy(env: Env, policy_id: u32) -> bool {
        let mut policies: Map<u32, InsurancePolicy> = env
            .storage()
            .instance()
            .get(&symbol_short!("POLICIES"))
            .unwrap_or_else(|| Map::new(&env));

        if let Some(mut policy) = policies.get(policy_id) {
            policy.active = false;
            policies.set(policy_id, policy);
            env.storage()
                .instance()
                .set(&symbol_short!("POLICIES"), &policies);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::{Ledger, LedgerInfo};
    use soroban_sdk::Env;

    fn create_test_env() -> Env {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set(LedgerInfo {
            timestamp: 1000000000, // Fixed timestamp for testing
            protocol_version: 20,
            sequence_number: 1,
            network_id: [0; 32],
            base_reserve: 10,
            min_temp_entry_ttl: 10,
            min_persistent_entry_ttl: 10,
            max_entry_ttl: 3110400,
        });
        env
    }

    #[test]
    fn test_create_policy_success() {
        let env = create_test_env();
        let contract_id = env.register_contract(None, Insurance);
        let client = InsuranceClient::new(&env, &contract_id);

        let name = String::from_str(&env, "Health Insurance");
        let coverage_type = String::from_str(&env, "health");
        let monthly_premium = 100;
        let coverage_amount = 10000;

        let policy_id =
            client.create_policy(&name, &coverage_type, &monthly_premium, &coverage_amount);

        assert_eq!(policy_id, 1);

        let policy = client.get_policy(&policy_id).unwrap();
        assert_eq!(policy.id, 1);
        assert_eq!(policy.name, name);
        assert_eq!(policy.coverage_type, coverage_type);
        assert_eq!(policy.monthly_premium, monthly_premium);
        assert_eq!(policy.coverage_amount, coverage_amount);
        assert!(policy.active);
        assert_eq!(policy.next_payment_date, 1000000000 + (30 * 86400));
    }

    #[test]
    #[should_panic(expected = "Monthly premium must be positive")]
    fn test_create_policy_zero_premium() {
        let env = create_test_env();
        let contract_id = env.register_contract(None, Insurance);
        let client = InsuranceClient::new(&env, &contract_id);

        let name = String::from_str(&env, "Health Insurance");
        let coverage_type = String::from_str(&env, "health");

        client.create_policy(&name, &coverage_type, &0, &10000);
    }

    #[test]
    #[should_panic(expected = "Monthly premium must be positive")]
    fn test_create_policy_negative_premium() {
        let env = create_test_env();
        let contract_id = env.register_contract(None, Insurance);
        let client = InsuranceClient::new(&env, &contract_id);

        let name = String::from_str(&env, "Health Insurance");
        let coverage_type = String::from_str(&env, "health");

        client.create_policy(&name, &coverage_type, &-100, &10000);
    }

    #[test]
    #[should_panic(expected = "Coverage amount must be positive")]
    fn test_create_policy_zero_coverage() {
        let env = create_test_env();
        let contract_id = env.register_contract(None, Insurance);
        let client = InsuranceClient::new(&env, &contract_id);

        let name = String::from_str(&env, "Health Insurance");
        let coverage_type = String::from_str(&env, "health");

        client.create_policy(&name, &coverage_type, &100, &0);
    }

    #[test]
    #[should_panic(expected = "Coverage amount must be positive")]
    fn test_create_policy_negative_coverage() {
        let env = create_test_env();
        let contract_id = env.register_contract(None, Insurance);
        let client = InsuranceClient::new(&env, &contract_id);

        let name = String::from_str(&env, "Health Insurance");
        let coverage_type = String::from_str(&env, "health");

        client.create_policy(&name, &coverage_type, &100, &-10000);
    }

    #[test]
    fn test_pay_premium_success() {
        let env = create_test_env();
        let contract_id = env.register_contract(None, Insurance);
        let client = InsuranceClient::new(&env, &contract_id);

        let name = String::from_str(&env, "Health Insurance");
        let coverage_type = String::from_str(&env, "health");
        let policy_id = client.create_policy(&name, &coverage_type, &100, &10000);

        let result = client.pay_premium(&policy_id);
        assert!(result);

        let policy = client.get_policy(&policy_id).unwrap();
        assert_eq!(policy.next_payment_date, 1000000000 + (30 * 86400));
    }

    #[test]
    fn test_pay_premium_inactive_policy() {
        let env = create_test_env();
        let contract_id = env.register_contract(None, Insurance);
        let client = InsuranceClient::new(&env, &contract_id);

        let name = String::from_str(&env, "Health Insurance");
        let coverage_type = String::from_str(&env, "health");
        let policy_id = client.create_policy(&name, &coverage_type, &100, &10000);

        // Deactivate policy
        client.deactivate_policy(&policy_id);

        let result = client.pay_premium(&policy_id);
        assert!(!result);
    }

    #[test]
    fn test_pay_premium_nonexistent_policy() {
        let env = create_test_env();
        let contract_id = env.register_contract(None, Insurance);
        let client = InsuranceClient::new(&env, &contract_id);

        let result = client.pay_premium(&999);
        assert!(!result);
    }

    #[test]
    fn test_get_policy_nonexistent() {
        let env = create_test_env();
        let contract_id = env.register_contract(None, Insurance);
        let client = InsuranceClient::new(&env, &contract_id);

        let policy = client.get_policy(&999);
        assert!(policy.is_none());
    }

    #[test]
    fn test_get_active_policies() {
        let env = create_test_env();
        let contract_id = env.register_contract(None, Insurance);
        let client = InsuranceClient::new(&env, &contract_id);

        // Create multiple policies
        let name1 = String::from_str(&env, "Health Insurance");
        let coverage_type1 = String::from_str(&env, "health");
        let policy_id1 = client.create_policy(&name1, &coverage_type1, &100, &10000);

        let name2 = String::from_str(&env, "Emergency Insurance");
        let coverage_type2 = String::from_str(&env, "emergency");
        let policy_id2 = client.create_policy(&name2, &coverage_type2, &200, &20000);

        let name3 = String::from_str(&env, "Life Insurance");
        let coverage_type3 = String::from_str(&env, "life");
        let policy_id3 = client.create_policy(&name3, &coverage_type3, &300, &30000);

        // Deactivate one policy
        client.deactivate_policy(&policy_id2);

        let active_policies = client.get_active_policies();
        assert_eq!(active_policies.len(), 2);

        // Check that only active policies are returned
        let mut ids = Vec::new(&env);
        for policy in active_policies.iter() {
            ids.push_back(policy.id);
        }
        assert!(ids.contains(&policy_id1));
        assert!(ids.contains(&policy_id3));
        assert!(!ids.contains(&policy_id2));
    }

    #[test]
    fn test_get_total_monthly_premium() {
        let env = create_test_env();
        let contract_id = env.register_contract(None, Insurance);
        let client = InsuranceClient::new(&env, &contract_id);

        // Create multiple policies
        let name1 = String::from_str(&env, "Health Insurance");
        let coverage_type1 = String::from_str(&env, "health");
        client.create_policy(&name1, &coverage_type1, &100, &10000);

        let name2 = String::from_str(&env, "Emergency Insurance");
        let coverage_type2 = String::from_str(&env, "emergency");
        client.create_policy(&name2, &coverage_type2, &200, &20000);

        let name3 = String::from_str(&env, "Life Insurance");
        let coverage_type3 = String::from_str(&env, "life");
        let policy_id3 = client.create_policy(&name3, &coverage_type3, &300, &30000);

        // Deactivate one policy
        client.deactivate_policy(&policy_id3);

        let total = client.get_total_monthly_premium();
        assert_eq!(total, 300); // 100 + 200 = 300
    }

    #[test]
    fn test_deactivate_policy_success() {
        let env = create_test_env();
        let contract_id = env.register_contract(None, Insurance);
        let client = InsuranceClient::new(&env, &contract_id);

        let name = String::from_str(&env, "Health Insurance");
        let coverage_type = String::from_str(&env, "health");
        let policy_id = client.create_policy(&name, &coverage_type, &100, &10000);

        let result = client.deactivate_policy(&policy_id);
        assert!(result);

        let policy = client.get_policy(&policy_id).unwrap();
        assert!(!policy.active);
    }

    #[test]
    fn test_deactivate_policy_nonexistent() {
        let env = create_test_env();
        let contract_id = env.register_contract(None, Insurance);
        let client = InsuranceClient::new(&env, &contract_id);

        let result = client.deactivate_policy(&999);
        assert!(!result);
    }

    #[test]
    fn test_multiple_policies_management() {
        let env = create_test_env();
        let contract_id = env.register_contract(None, Insurance);
        let client = InsuranceClient::new(&env, &contract_id);

        // Create 5 policies
        let mut policy_ids = Vec::new(&env);
        let policy_names = [
            String::from_str(&env, "Policy 1"),
            String::from_str(&env, "Policy 2"),
            String::from_str(&env, "Policy 3"),
            String::from_str(&env, "Policy 4"),
            String::from_str(&env, "Policy 5"),
        ];
        let coverage_type = String::from_str(&env, "health");

        for i in 0..5 {
            let premium = ((i + 1) as i128) * 100;
            let coverage = ((i + 1) as i128) * 10000;
            let policy_id =
                client.create_policy(&policy_names[i], &coverage_type, &premium, &coverage);
            policy_ids.push_back(policy_id);
        }

        // Pay premium for all policies
        for policy_id in policy_ids.iter() {
            assert!(client.pay_premium(&policy_id));
        }

        // Deactivate 2 policies
        client.deactivate_policy(&policy_ids.get(1).unwrap());
        client.deactivate_policy(&policy_ids.get(3).unwrap());

        // Check active policies
        let active_policies = client.get_active_policies();
        assert_eq!(active_policies.len(), 3);

        // Check total premium (1+3+5)*100 = 900
        let total = client.get_total_monthly_premium();
        assert_eq!(total, 900);
    }

    #[test]
    fn test_large_amounts() {
        let env = create_test_env();
        let contract_id = env.register_contract(None, Insurance);
        let client = InsuranceClient::new(&env, &contract_id);

        let name = String::from_str(&env, "Premium Insurance");
        let coverage_type = String::from_str(&env, "premium");
        let monthly_premium = i128::MAX / 2; // Very large amount
        let coverage_amount = i128::MAX / 2;

        let policy_id =
            client.create_policy(&name, &coverage_type, &monthly_premium, &coverage_amount);

        let policy = client.get_policy(&policy_id).unwrap();
        assert_eq!(policy.monthly_premium, monthly_premium);
        assert_eq!(policy.coverage_amount, coverage_amount);
    }
}
