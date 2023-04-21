#![no_std]

multiversx_sc::imports!();

mod validation;
mod tokens;
mod storage;

use validation::Signature;

#[multiversx_sc::contract]
pub trait GameMinter: 
    validation::ValidationModule + 
    tokens::TokensModule + 
    storage::StorageModule
{

    #[init]
    fn init(
        &self, 
        signer: ManagedAddress, 
        platform_currency: TokenIdentifier, 
        planet_price: BigUint,
        krogan: ManagedAddress, 
        rewards_pool: ManagedAddress, 
    ) {
        self.signer().set_if_empty(&signer);
        self.platform_currency().set_if_empty(platform_currency);
        self.planet_price().set_if_empty(planet_price);
        self.krogan().set_if_empty(krogan);
        self.rewards_pool().set_if_empty(rewards_pool);
    }

    #[only_owner]
    #[endpoint(mintTokens)]
    fn mint_tokens(
        &self,
        token_key: ManagedBuffer,
        amount: BigUint,
    ) {
        let caller = self.blockchain().get_caller();
        self.mint_and_send_token(&caller, &token_key, &amount);
    }

    #[endpoint(claimTokens)]
    fn claim_tokens(
        &self,
        _starbase_nonce: u32, // only used by backend
        player_nonce: u32,
        data: MultiValueEncoded<MultiValue4<
            u32, 
            ManagedBuffer<Self::Api>, 
            BigUint<Self::Api>, 
            Signature<Self::Api>
        >>,
    ) {
        let caller = self.blockchain().get_caller();

        for user_data in data.into_iter() {
            let (index, token_key, amount, signature) = user_data.into_tuple();
        
            self.verify_signature(&index, &caller, &token_key, &amount, &signature);

            // A limit in case of hacking
            require!(amount < BigUint::from(ManagedBuffer::from(b"180000000000000000000")), "Invalid amount."); // max 180
            require!(index == player_nonce, "Something went wrong.");
            require!(self.tokens_claimed(&caller, &index).get() == false, "Already claimed these materials.");
            require!(self.material_token_id(&token_key).is_empty() == false, "Invalid material.");
            
            self.mint_and_send_token(&caller, &token_key, &amount);
        }
        self.tokens_claimed(&caller, &player_nonce).set(true);
    }

    #[payable("*")]
    #[endpoint(claimPlanet)]
    fn claim_planet(
        &self,
        nonce: u32,
        token_id: TokenIdentifier<Self::Api>, 
        amount: BigUint<Self::Api>, 
        signature: Signature<Self::Api>
    ) {
        let caller = self.blockchain().get_caller();
        require!(self.planet_claimed(&self.planet_token_id().get(), &nonce).get() == false, "Planet already claimed.");
        require!(token_id == self.planet_token_id().get(), "Invalid data.");
        require!(amount == BigUint::from(1u32), "Invalid data.");

        let payment = self.call_value().single_esdt();
        require!(payment.token_identifier == self.platform_currency().get(), "Invalid payment.");
        require!(payment.token_nonce == 0, "Invalid payment.");
        require!(payment.amount == self.planet_price().get(), "Invalid payment.");

        self.planet_claimed(&self.planet_token_id().get(), &nonce).set(true);

        self.split_game_revenue_and_send(&payment.amount);

        self.verify_signature(&nonce, &caller, &token_id.as_managed_buffer(), &amount, &signature);
        self.send().direct(&caller, &EgldOrEsdtTokenIdentifier::esdt(token_id), nonce.into(), &amount);
    }

    #[only_owner]
    #[endpoint(withdrawProfits)]
    fn withdraw_profits(
        &self, 
        opt_payment_token: OptionalValue<EgldOrEsdtTokenIdentifier>
    ) {
        let payment_token = opt_payment_token
            .into_option()
            .unwrap_or_else(|| EgldOrEsdtTokenIdentifier::egld());
        let amount = self.blockchain().get_sc_balance(&&payment_token, 0);
        self.send().direct(&self.blockchain().get_owner_address(), &payment_token, 0, &amount);
    }

}
