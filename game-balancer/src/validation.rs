elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use elrond_wasm::api::ED25519_SIGNATURE_BYTE_LEN;

pub type Timestamp = u64;

// index + caller + token + amount
// 4 + 32 + 32 + (4 + 32) = 108, with some extra for high BigUint values
const MAX_DATA_LEN: usize = 120;

pub type Signature<M> = ManagedByteArray<M, ED25519_SIGNATURE_BYTE_LEN>;

#[elrond_wasm::module]
pub trait ValidationModule: crate::storage::StorageModule {
    
    fn verify_signature(
        &self,
        timestamp: &Timestamp,
        caller: &ManagedAddress,
        faction: &ManagedAddress,
        signature: &Signature<Self::Api>,
    ) {
        let mut data = ManagedBuffer::new();
        let _ = timestamp.dep_encode(&mut data);
        data.append(caller.as_managed_buffer());
		data.append(faction.as_managed_buffer());

        let signer: ManagedAddress = self.signer().get();
        let valid_signature = self.crypto().verify_ed25519_legacy_managed::<MAX_DATA_LEN>(
            signer.as_managed_byte_array(),
            &data,
            signature,
        );
		require!(self.signed(&caller, &timestamp).is_empty(), "Invalid signature!");
        require!(valid_signature, "Invalid signature");

		self.signed(&caller, &timestamp).set(true);
    }
}