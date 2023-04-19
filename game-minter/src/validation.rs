multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use multiversx_sc::api::ED25519_SIGNATURE_BYTE_LEN;

pub type Index = u32;

// index + caller + token + amount
// 4 + 32 + 32 + (4 + 32) = 108, with some extra for high BigUint values
const MAX_DATA_LEN: usize = 120;

pub type Signature<M> = ManagedByteArray<M, ED25519_SIGNATURE_BYTE_LEN>;

#[multiversx_sc::module]
pub trait ValidationModule: crate::storage::StorageModule {
    fn verify_signature(
        &self,
        index: &Index,
        caller: &ManagedAddress,
        token_key: &ManagedBuffer<Self::Api>,
        amount: &BigUint<Self::Api>,
        signature: &Signature<Self::Api>,
    ) {
        let mut data = ManagedBuffer::new();
        let _ = index.dep_encode(&mut data);
        data.append(caller.as_managed_buffer());
		data.append(token_key);
        let _ = amount.dep_encode(&mut data);

        let signer: ManagedAddress = self.signer().get();
        let valid_signature = self.crypto().verify_ed25519_legacy_managed::<MAX_DATA_LEN>(
            signer.as_managed_byte_array(),
            &data,
            signature,
        );

        require!(valid_signature, "Invalid signature");
    }
    
}