elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use crate::models::*;
use crate::validation::Signature;

#[elrond_wasm::module]
pub trait RewardsModule: 
    crate::storage::StorageModule +
    crate::validation::ValidationModule
{

    #[payable("*")]
    #[endpoint(addRewardsCheckpoint)]
    fn add_rewards_checkpoint(
        &self,
        root_hash: ManagedHash<Self::Api>,
        total_nft_supply: BigUint
    ) {
        require!(self.rewards_checkpoints(&root_hash).is_empty(), "Checkpoint already exists");
        let caller = self.blockchain().get_caller();
        require!(self.whitelisted(caller.clone()).get(), "Not allowed to deposit!");

        let (reward_token, reward_nonce, reward_supply) = self.call_value().payment_as_tuple();

        require!(reward_supply > 0, "Amount must be higher than 0");

        let checkpoint = RewardsCheckpoint {
            total_nft_supply,
            reward_token,
            reward_supply,
            reward_nonce,
        };
        self.rewards_checkpoints(&root_hash).set(&checkpoint);
        self.rewards_owner(&root_hash).set(caller);
    }

    #[endpoint(removeRewardsCheckpoint)]
    fn remove_rewards_checkpoint(
        &self,
        root_hash: ManagedHash<Self::Api>,
        nft_supply_left: u32,
        signature: Signature<Self::Api>
    ) {
        require!(!self.rewards_checkpoints(&root_hash).is_empty(), "Checkpoint does not exist");
        let caller = self.blockchain().get_caller();

        if self.rewards_owner(&root_hash).is_empty() == false {
            let owner = self.rewards_owner(&root_hash).get();
            require!(owner == caller, "Not allowed to remove rewards from this project");
        } else {
            // For previous deposits - some does not have `rewards_owner`
            // All previous deposits are made by the owner
            self.blockchain().check_caller_is_owner();
        }

        self.verify_signature(&caller, &root_hash, &nft_supply_left, &signature);

        require!(!self.rewards_checkpoints(&root_hash).is_empty(), "Invalid root hash");
        let checkpoint: RewardsCheckpoint<Self::Api> = self.rewards_checkpoints(&root_hash).get();

        let reward_amount = self.calculate_reward_amount(
            checkpoint.reward_supply,
            nft_supply_left,
            checkpoint.total_nft_supply,
        );

        // Clear all data linked to the hash
        self.rewards_checkpoints(&root_hash).clear();
        self.rewards_owner(&root_hash).clear();

        // Return rewards to owner
        self.send().direct(&caller, &checkpoint.reward_token, checkpoint.reward_nonce, &reward_amount, &[]);
    }

    fn calculate_reward_amount(
        &self,
        rewards_supply: BigUint,
        user_nft_amount: u32,
        total_nft_supply: BigUint,
    ) -> BigUint {
        (rewards_supply * BigUint::from(user_nft_amount.clone())) / total_nft_supply
    }

}