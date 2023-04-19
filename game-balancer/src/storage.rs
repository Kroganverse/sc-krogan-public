elrond_wasm::imports!();
elrond_wasm::derive_imports!();

// use crate::models::*;

#[elrond_wasm::module]
pub trait StorageModule {

	// SETTERS

	#[only_owner]
    #[endpoint(changeSigner)]
    fn change_signer(&self, new_signer: ManagedAddress) {
        self.signer().set(&new_signer);
    }

	// PROXY

	#[proxy]
    fn faction_contract(&self, sc_address: ManagedAddress) -> game_faction::Proxy<Self::Api>;

    // STORAGE

	#[storage_mapper("signer")]
    fn signer(&self) -> SingleValueMapper<ManagedAddress>;

	#[storage_mapper("signed")]
    fn signed(&self, address: &ManagedAddress, timestamp: &u64) -> SingleValueMapper<bool>;
    
    #[view(getFactions)]
    #[storage_mapper("factions")]
    fn factions(&self) -> SingleValueMapper<ManagedVec<ManagedAddress>>;

	#[view(getUserFaction)]
    #[storage_mapper("userFaction")]
    fn user_faction(&self, address: &ManagedAddress) -> SingleValueMapper<ManagedAddress>;

}