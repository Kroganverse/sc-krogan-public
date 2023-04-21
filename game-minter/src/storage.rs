multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait StorageModule {

    #[only_owner]
    #[endpoint(changeSigner)]
    fn change_signer(&self, new_signer: ManagedAddress) {
        self.signer().set(&new_signer);
    }

    #[only_owner]
    #[endpoint(changeKroganAddress)]
    fn change_krogan_address(&self, address: ManagedAddress) {
        self.krogan().set(&address);
    }

    #[only_owner]
    #[endpoint(changeRewardsPoolAddress)]
    fn change_rewards_pool_address(&self, address: ManagedAddress) {
        self.rewards_pool().set(&address);
    }

    #[only_owner]
    #[endpoint(changePlanetTokenId)]
    fn change_planet_token_id(&self, new_token_id: TokenIdentifier) {
        self.planet_token_id().set(&new_token_id);
    }

    #[view(getPlanetClaimed)]
    fn get_planet_claimed(&self, nonce: u32) -> bool {
        self.planet_claimed(&self.planet_token_id().get(), &nonce).get()
    }
 
    #[only_owner]
    #[endpoint(resetPlanetClaim)]
    fn reset_planet_claim(&self, nonce: u32) {
        self.planet_claimed(&self.planet_token_id().get(), &nonce).clear();
    }
    
	#[storage_mapper("signer")]
    fn signer(&self) -> SingleValueMapper<ManagedAddress>;

	#[storage_mapper("krogan")]
	fn krogan(&self) -> SingleValueMapper<ManagedAddress>;

	#[storage_mapper("rewardsPool")]
	fn rewards_pool(&self) -> SingleValueMapper<ManagedAddress>;

    #[endpoint(getTokensClaimed)]
    #[storage_mapper("tokensClaimed")]
    fn tokens_claimed(
        &self,
        user: &ManagedAddress<Self::Api>,
        index: &u32,
    ) -> SingleValueMapper<bool>;

	#[view(getMaterialTokenId)]
    #[storage_mapper("materialTokenId")]
    fn material_token_id(&self, key: &ManagedBuffer) -> SingleValueMapper<TokenIdentifier>;
    
	#[storage_mapper("planetTokenId")]
    fn planet_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[storage_mapper("planetClaimed")]
    fn planet_claimed(&self, token: &TokenIdentifier<Self::Api>, nonce: &u32) -> SingleValueMapper<bool>;

    #[view(getPlanetPrice)]
    #[storage_mapper("planetPrice")]
    fn planet_price(&self) -> SingleValueMapper<BigUint>;

    #[view(getPlatformCurrency)]
    #[storage_mapper("platformCurrency")]
    fn platform_currency(&self) -> SingleValueMapper<TokenIdentifier>;

}