multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait TokensModule: crate::storage::StorageModule {
    
	#[only_owner]
    #[payable("EGLD")]
    #[endpoint(issueToken)]
    fn issue_token(
        &self,
        #[payment] issue_cost: BigUint<Self::Api>,
        token_name: ManagedBuffer<Self::Api>,
        token_ticker: ManagedBuffer<Self::Api>,
        opt_replace: OptionalValue<bool>,
    ) {
        let replace = opt_replace
            .into_option()
            .unwrap_or_else(|| false);

        if replace == false {
            require!(self.material_token_id(&token_name).is_empty(), "Token already issued.");
        }

        self
            .send()
            .esdt_system_sc_proxy()
            .issue_fungible(
                issue_cost,
                &token_name,
                &token_ticker,
                &BigUint::from(0u32),
                FungibleTokenProperties {
					num_decimals: 18,
                    can_freeze: true,
                    can_wipe: true,
                    can_pause: true,
                    can_mint: true,
                    can_burn: true,
                    can_change_owner: true,
                    can_upgrade: true,
                    can_add_special_roles: true
                },
            )
            .async_call()
            .with_callback(self.callbacks().issue_callback(token_name))
            .call_and_exit()
    }

	#[callback]
    fn issue_callback(
		&self, 
		key: ManagedBuffer<Self::Api>, 
		#[call_result] result: ManagedAsyncCallResult<TokenIdentifier>
	) {
        match result {
            ManagedAsyncCallResult::Ok(token_id) => {
                self.material_token_id(&key).set(token_id.clone());
				self.set_local_roles(token_id);
            },
            ManagedAsyncCallResult::Err(_) => {
                let caller = self.blockchain().get_owner_address();
                let (token_id, returned_tokens) = self.call_value().egld_or_single_fungible_esdt();
                if token_id.is_egld() && returned_tokens > BigUint::zero() {
                    self.send().direct(&caller, &token_id, 0, &returned_tokens);
                }
            },
        }
    }

    fn set_local_roles(&self, token_id: TokenIdentifier<Self::Api>) {

        let address = self.blockchain().get_sc_address();

        self
            .send()
            .esdt_system_sc_proxy()
            .set_special_roles(
                &address,
                &token_id,
                (&[
                    EsdtLocalRole::Mint, 
                    EsdtLocalRole::Burn
                ][..]).into_iter().cloned(),
            )
            .async_call()
            .call_and_exit()
    }

	fn mint_and_send_token(
		&self,
		address: &ManagedAddress,
		token_key: &ManagedBuffer,
		amount: &BigUint
	) {
		let token_id = self.material_token_id(&token_key).get();

		self.send().esdt_local_mint(&token_id, 0, &amount);
		self.send().direct(&address, &EgldOrEsdtTokenIdentifier::esdt(token_id), 0, &amount);
	}

    /**
     * All transactions between companies/players and the game
     */
    fn split_game_revenue_and_send(&self, amount: &BigUint) {
        self.send().direct_esdt(&self.krogan().get(), &self.platform_currency().get(), 0, &(amount / 2u32));
        self.send().direct_esdt(&self.rewards_pool().get(), &self.platform_currency().get(), 0, &(amount / 2u32));
    }

}