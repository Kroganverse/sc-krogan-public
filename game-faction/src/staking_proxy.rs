elrond_wasm::imports!();

#[elrond_wasm::proxy]
pub trait StakingContract {
	
	#[view(currentUserData)]
    fn current_user_data(
		&self, 
		address: ManagedAddress
	) -> MultiValue6<BigUint, BigUint, u64, bool, BigUint, BigUint>;
}