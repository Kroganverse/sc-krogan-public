elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use crate::staking_proxy;
use crate::models::*;

#[elrond_wasm::module]
pub trait StorageModule {

	// PROXY

	#[proxy]
    fn staking_contract(&self, sc_address: ManagedAddress) -> staking_proxy::Proxy<Self::Api>;

    // STORAGE
    
	#[view(getBalancer)]
    #[storage_mapper("balancer")]
    fn balancer(&self) -> SingleValueMapper<ManagedAddress>;
    
    #[storage_mapper("platformCurrency")]
    fn platform_currency(&self) -> SingleValueMapper<TokenIdentifier>;

	#[view(getStakingPool)]
    #[storage_mapper("stakingPool")]
    fn staking_pool(&self) -> SingleValueMapper<ManagedAddress>;

	#[storage_mapper("krogan")]
    fn krogan(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("rewardsPool")]
    fn rewards_pool(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("minimumStake")]
    fn minimum_stake(&self) -> SingleValueMapper<BigUint>;

	// Users

	#[view(getUserCount)]
    #[storage_mapper("userCount")]
    fn user_count(&self) -> SingleValueMapper<u32>;

    #[view(isRegistered)]
    #[storage_mapper("isRegistered")]
    fn is_registered(&self, address: &ManagedAddress) -> SingleValueMapper<bool>;

	// Dao

	#[view(getDaoIndex)]
    #[storage_mapper("daoIndex")]
    fn dao_index(&self) -> SingleValueMapper<u32>;

    #[view(getDaoItem)]
    #[storage_mapper("daoItem")]
    fn dao_item(&self, id: &u32) -> SingleValueMapper<DaoItem<Self::Api>>;

    #[view(getUserVoted)]
    #[storage_mapper("userVoteWeight")]
    fn user_vote_weight(&self, address: &ManagedAddress, id: &u32) -> SingleValueMapper<BigUint>;

	#[view(getProposalTypeCost)]
    #[storage_mapper("proposalTypeCost")]
    fn proposal_type_cost(&self, proposal_type: &ManagedBuffer) -> SingleValueMapper<BigUint>;

}