#![no_std]

elrond_wasm::imports!();

mod validation;
mod storage;
mod rewards;
mod views;
pub mod models;

use models::*;
use crate::validation::Signature;

#[elrond_wasm::contract]
pub trait NftAirdrop: 
    validation::ValidationModule +
    rewards::RewardsModule + 
    views::ViewsModule +
    storage::StorageModule + 
{

    #[init]
    fn init(&self, signer: ManagedAddress) {
        self.signer().set_if_empty(&signer);
    }

    #[endpoint(claimRewards)]
    fn claim_rewards(
        &self,
        #[var_args] data: MultiValueEncoded<MultiValue3<ManagedHash<Self::Api>, u32, Signature<Self::Api>>>,
    ) {
        let caller = self.blockchain().get_caller();

        let mut egld_payment_amount = BigUint::zero();
        let mut output_payments = ManagedVec::new();
        let mut last_payment = EsdtTokenPayment::no_payment();

        for user_data in data.into_iter() {
            let (hash, 
                amount, 
                signature) = user_data.into_tuple();

            // Check if owner is present & whitelisted
            if self.rewards_owner(&hash).is_empty() == false {
                let owner = self.rewards_owner(&hash).get();
                require!(self.whitelisted(owner).get(), "Not allowed to claim rewards from this project!");
            }

            self.verify_signature(&caller, &hash, &amount, &signature);
            require!(!self.rewards_claimed(&caller, &hash).get(), "Already claimed rewards for this week");

            let checkpoint_mapper = self.rewards_checkpoints(&hash);
            require!(!checkpoint_mapper.is_empty(), "Invalid root hash");
            let checkpoint: RewardsCheckpoint<Self::Api> = checkpoint_mapper.get();

            let reward_amount = self.calculate_reward_amount(
                checkpoint.reward_supply,
                amount,
                checkpoint.total_nft_supply,
            );

            self.rewards_claimed(&caller, &hash).set(&true);

            if checkpoint.reward_token == TokenIdentifier::egld() {
                egld_payment_amount += reward_amount;
            } else {
                if checkpoint.reward_token != last_payment.token_identifier && last_payment.token_type != EsdtTokenType::Invalid {
                    output_payments.push(last_payment.clone());
                    last_payment = EsdtTokenPayment::no_payment();
                }

                if checkpoint.reward_nonce != 0 {
                    last_payment = EsdtTokenPayment::new(checkpoint.reward_token, checkpoint.reward_nonce, reward_amount);
                    output_payments.push(last_payment.clone());
                    last_payment = EsdtTokenPayment::no_payment();
                } else {
                    if checkpoint.reward_token == last_payment.token_identifier {
                        last_payment.amount += reward_amount;
                    } else {
                        last_payment = EsdtTokenPayment::new(checkpoint.reward_token, checkpoint.reward_nonce, reward_amount);
                    }
                }
            }
        }
        if last_payment.token_type != EsdtTokenType::Invalid {
            output_payments.push(last_payment.clone());
        }
        if egld_payment_amount > 0 {
            self.send().direct_egld(&caller, &egld_payment_amount, &[]);
        }
        if !output_payments.is_empty() {
            self.send().direct_multi(&caller, &output_payments, &[]);
        }

    }

    #[only_owner]
    #[endpoint(withdrawAll)]
    fn withdraw_all(
        &self,
        #[var_args] opt_token_identifier: OptionalValue<TokenIdentifier>,
        #[var_args] opt_token_nonce: OptionalValue<u64>
    ) {
        let token_identifier = opt_token_identifier
            .into_option()
            .unwrap_or_else(|| TokenIdentifier::egld());

        let token_nonce = if token_identifier.is_egld() {
            0
        } else {
            opt_token_nonce
                .into_option()
                .unwrap_or_default()
        };

        let owner = self.blockchain().get_owner_address();
        let balance = self.blockchain().get_sc_balance(&token_identifier, token_nonce);
        require!(balance > 0, "Nothing to withdraw.");
        self.send().direct(&owner, &token_identifier, token_nonce, &balance, b"Emergency withdraw");
    }
    
}
