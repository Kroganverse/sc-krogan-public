#![no_std]

elrond_wasm::imports!();

// modules
mod storage;
mod validation;

use validation::Signature;
use game_faction::ProxyTrait as _;

#[elrond_wasm::contract]
pub trait BalancerContract: 
    storage::StorageModule +
    validation::ValidationModule
{
    #[init]
    fn init(
        &self, 
        factions: &ManagedVec<ManagedAddress>
    ) {
        self.factions().set_if_empty(factions);

        let sc_address = self.blockchain().get_sc_address();
        for item in factions {
            let _: IgnoreValue = self.faction_contract(item)
                .register_balancer(sc_address.clone())
                .execute_on_dest_context();
        }
    }

    #[only_owner]
    #[endpoint(registerProposalType)]
    fn register_proposal_type(&self, proposal_type: &ManagedBuffer, cost: &BigUint) {
        let factions = self.factions().get();
        for item in &factions {
            let _: IgnoreValue = self.faction_contract(item.clone())
                .register_proposal_type(proposal_type.clone(), cost.clone())
                .execute_on_dest_context();
        }
    }

    #[endpoint(registerUser)]
    fn register_user(
        &self, 
        faction_address: ManagedAddress, 
        timestamp: u64,
        signature: Signature<Self::Api>
    ) {
        let caller = self.blockchain().get_caller();
        let time = self.blockchain().get_block_timestamp();
        require!(time - timestamp < 3600, "Invalid request!");

        // Remove from old faction. This enables faster testing times
        if self.user_faction(&caller).is_empty() == false {
            let _: IgnoreValue = self.faction_contract(self.user_faction(&caller).get())
            .remove_user(&caller)
            .execute_on_dest_context();
        }

        self.verify_signature(&timestamp, &caller, &faction_address, &signature);

        let _: IgnoreValue = self.faction_contract(faction_address.clone())
            .register_user(&caller)
            .execute_on_dest_context();
        
        self.user_faction(&caller).set(faction_address);
    }

    #[endpoint(swapUser)]
    fn swap_user(
        &self, 
        with_address: ManagedAddress, 
        timestamp: u64,
        signature: Signature<Self::Api>
    ) {
        let caller = self.blockchain().get_caller();
        let time = self.blockchain().get_block_timestamp();
        require!(time - timestamp < 3600, "Invalid request!");
        
        self.verify_signature(&timestamp, &caller, &with_address, &signature);

        let faction1 = self.user_faction(&caller).get();
        let faction2 = self.user_faction(&with_address).get();
        require!(faction1 != faction2, "Cannot swap with the same faction");

        let _: IgnoreValue = self.faction_contract(faction1)
            .swap_user(&caller, &with_address)
            .execute_on_dest_context();
        
        let _: IgnoreValue = self.faction_contract(faction2)
            .swap_user(&with_address, &caller)
            .execute_on_dest_context();
        
    }

    #[only_owner]
    #[endpoint(removeUser)]
    fn remove_user(
        &self, 
        address: ManagedAddress
    ) {
        let faction = self.user_faction(&address).get();
        
        let _: IgnoreValue = self.faction_contract(faction)
            .remove_user(&address)
            .execute_on_dest_context();
        
    }
}
