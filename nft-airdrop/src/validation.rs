elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use crate::models::*;

const SIGNATURE_LEN: usize = 64;
const MAX_DATA_LEN: usize = 120;

pub type Signature<M> = ManagedByteArray<M, SIGNATURE_LEN>;

#[elrond_wasm::module]
pub trait ValidationModule: crate::storage::StorageModule {
    
    fn verify_signature(
        &self,
        address: &ManagedAddress<Self::Api>,
        root_hash: &ManagedHash<Self::Api>,
        user_nft_amount: &u32,
        signature: &Signature<Self::Api>,
    ) {
        let data = self.create_hash(&address, root_hash, user_nft_amount);

        let signer: ManagedAddress = self.signer().get();
        let valid_signature = self.crypto().verify_ed25519_managed::<MAX_DATA_LEN>(
            &signer.as_managed_byte_array(),
            &data,
            &signature
        );
        require!(valid_signature, "Invalid signature");
    }

    fn create_hash(&self, address: &ManagedAddress, hash: &ManagedHash<Self::Api>, amount: &u32) -> ManagedBuffer {

        let mut buffer = ManagedBuffer::new();
        buffer.append(address.as_managed_buffer());
        buffer.append(hash.as_managed_buffer());
        buffer.append(&ManagedBuffer::from(b"_"));
        buffer.append(&self.decimal_to_ascii(amount.clone()));

        buffer
    }

	fn decimal_to_ascii(&self, mut number: u32) -> ManagedBuffer {
        const MAX_NUMBER_CHARACTERS: usize = 10;
        const ZERO_ASCII: u8 = b'0';

        let mut as_ascii = [0u8; MAX_NUMBER_CHARACTERS];
        let mut nr_chars = 0;

        loop {
            unsafe {
                let reminder: u8 = (number % 10).try_into().unwrap_unchecked();
                number /= 10;

                as_ascii[nr_chars] = ZERO_ASCII + reminder;
                nr_chars += 1;
            }

            if number == 0 {
                break;
            }
        }

        let slice = &mut as_ascii[..nr_chars];
        slice.reverse();

        ManagedBuffer::new_from_bytes(slice)
    }
}