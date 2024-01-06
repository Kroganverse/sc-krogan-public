#![no_std]

elrond_wasm::imports!();

mod staking_proxy;
mod storage;
pub mod models;

use models::*;
use crate::staking_proxy::ProxyTrait as _;

/// An empty contract. To be used as a template when starting a new contract from scratch.
#[elrond_wasm::contract]
pub trait FactionContract: storage::StorageModule {
    #[init]
    fn init(
        &self,
        staking_pool: ManagedAddress,
        platform_currency: TokenIdentifier,
        krogan: ManagedAddress, 
        rewards_pool: ManagedAddress, 
        minimum_stake: BigUint,
    ) {
        self.staking_pool().set_if_empty(staking_pool);
        self.platform_currency().set_if_empty(platform_currency);

        self.krogan().set_if_empty(krogan);
        self.rewards_pool().set_if_empty(rewards_pool);
        self.minimum_stake().set_if_empty(minimum_stake);
    }

    // This endpoint is called by the balancer itself on deploy
    #[endpoint(registerBalancer)]
    fn register_balancer(&self, address: ManagedAddress) {
        self.balancer().set_if_empty(address); // can set only once
    }

    #[endpoint(registerProposalType)]
    fn register_proposal_type(&self, proposal_type: ManagedBuffer, cost: BigUint) {
        let caller = self.blockchain().get_caller();
        require!(caller == self.balancer().get(), "Not authorized!");

        self.proposal_type_cost(&proposal_type).set(cost);
    }

    #[endpoint(registerUser)]
    fn register_user(&self, address: &ManagedAddress) {
        let caller = self.blockchain().get_caller();
        require!(caller == self.balancer().get(), "Not authorized!");

        self.is_registered(&address).set(true);
        self.user_count().update(|val| *val += 1);
    }

    #[endpoint(swapUser)]
    fn swap_user(&self, address: &ManagedAddress, with_address: &ManagedAddress) {
        let caller = self.blockchain().get_caller();
        require!(caller == self.balancer().get(), "Not authorized!");

        self.is_registered(&address).clear();
        self.is_registered(&with_address).set(true);
    }

    #[endpoint(removeUser)]
    fn remove_user(&self, address: &ManagedAddress) {
        let caller = self.blockchain().get_caller();
        require!(caller == self.balancer().get(), "Not authorized!");

        self.is_registered(&address).clear();
    }

    #[endpoint]
    fn propose(
        &self,
        description: ManagedBuffer,
        proposal_type: ManagedBuffer,
        args: ManagedBuffer, // in the form: type:value;
    ) -> u32 {
        let caller = self.blockchain().get_caller();
        require!(self.is_registered(&caller).get(), "Not part of this faction!");

        // Check staking
        let caller_staking = self.get_staking_amount(&caller);
        let one_kro = BigUint::from(1000000000000000000u64);
        let caller_staking_parsed = caller_staking.clone() / one_kro;
        require!(caller_staking >= self.minimum_stake().get(), "Cannot propose without 30000 KRO active staking, currently {} KRO", caller_staking_parsed);

        // Update index
        let index = self.dao_index().get() + 1;
        self.dao_index().set(index);

        // Create Dao Item
        let timestamp = self.blockchain().get_block_timestamp();
        let item = DaoItem {
            id: index,
            owner: caller.clone(),
            deadline: timestamp + 3 * 24 * 3600, // 3D
            proposal: description,
            proposal_type: proposal_type,
            args: args,
            up_votes: caller_staking.clone(),
            down_votes: BigUint::zero(),
            abstain_votes: BigUint::zero(),
            votes: 1,
            status: ProposalStatus::Pending,
        };

        self.dao_item(&index).set(item);
        self.user_vote_weight(&caller, &index).set(caller_staking);

        index
    }

    #[endpoint]
    fn vote(
        &self,
        proposal_id: &u32,
        vote: VoteType,    
    ) {
        let caller = self.blockchain().get_caller();
        let dao_item = self.dao_item(&proposal_id).get();
        require!(self.is_registered(&caller).get(), "Not part of this faction!");
        require!(self.user_vote_weight(&caller, &proposal_id).is_empty(), "Already voted.");
        require!(dao_item.deadline > self.blockchain().get_block_timestamp(), "Proposal expired.");

        // Check staking
        let caller_staking = self.get_staking_amount(&caller);
        require!(caller_staking > BigUint::zero(), "Cannot vote without active KRO staking");

        match vote {
            VoteType::UpVote => {
                self.dao_item(&proposal_id).update(|item| {
                    item.up_votes += &caller_staking.clone();
                    item.votes += 1;
                });
            },
            VoteType::DownVote => {
                self.dao_item(&proposal_id).update(|item| {
                    item.down_votes += &caller_staking.clone();
                    item.votes += 1;
                });
            },
            VoteType::AbstainVote => {
                self.dao_item(&proposal_id).update(|item| {
                    item.abstain_votes += &caller_staking.clone();
                    item.votes += 1;
                });
            },
        }

        self.user_vote_weight(&caller, &proposal_id).set(caller_staking.clone());
    }


    #[endpoint(concludeProposal)]
    fn conclude_proposal(
        &self,
        proposal_id: u32,
    ) {
        let caller = self.blockchain().get_caller();
        let mut dao_item = self.dao_item(&proposal_id).get();
        require!(self.is_registered(&caller).get(), "Not part of this faction!");
        require!(dao_item.deadline < self.blockchain().get_block_timestamp(), "Proposal is active for 3 days.");

        let balance = self.blockchain().get_sc_balance(
            &EgldOrEsdtTokenIdentifier::esdt(self.platform_currency().get()), 0
        );

        let total_vote_power = dao_item.clone().abstain_votes + dao_item.clone().up_votes + dao_item.clone().down_votes;
        let up_votes = BigUint::from(10000u32) * dao_item.clone().up_votes / total_vote_power.clone();
        let down_votes = BigUint::from(10000u32) * dao_item.clone().down_votes / total_vote_power;

        if up_votes > down_votes {
            let cost = self.proposal_type_cost(&dao_item.proposal_type).get();
            if cost > BigUint::zero() {
                require!(balance >= cost.clone(), "The faction does not have the funds for this action.");
                self.split_game_revenue_and_send(&cost);
            }
            dao_item.status = ProposalStatus::Succeeded;
        } else {
            dao_item.status = ProposalStatus::Failed;
        }

        self.dao_item(&proposal_id).set(dao_item);
    }

    #[endpoint(removeProposal)]
    fn remove_proposal(
        &self,
        proposal_id: u32,
    ) {
        let caller = self.blockchain().get_caller();
        let dao_item = self.dao_item(&proposal_id).get();
        require!(dao_item.owner == caller, "Not authorized");
        require!(dao_item.votes == 1, "People already voted");

        self.dao_item(&proposal_id).clear();
    }

    // HELPERS

    fn split_game_revenue_and_send(&self, amount: &BigUint) {
        self.send().direct_esdt(&self.krogan().get(), &self.platform_currency().get(), 0, &(amount / 2u32));
        self.send().direct_esdt(&self.rewards_pool().get(), &self.platform_currency().get(), 0, &(amount / 2u32));
    }

    fn get_staking_amount(&self, address: &ManagedAddress) -> BigUint {
        let result: MultiValue6<BigUint, BigUint, u64, bool, BigUint, BigUint> 
            = self.staking_contract(self.staking_pool().get())
                .current_user_data(address)
                .execute_on_dest_context();
        
        let (
            _, _, _, _,
            earned,
            user_balance
        ) = result.into_tuple();

        return user_balance + earned
    }

}
