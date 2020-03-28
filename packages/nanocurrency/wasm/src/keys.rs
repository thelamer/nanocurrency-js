use crate::base32;
use crate::custom_ed25519;
use crate::utils::ToBigEndian;
use crypto::blake2b::Blake2b;
use crypto::digest::Digest;

/// A Nano seed
pub struct Seed(pub [u8; 32]);
/// A Nano account private key
pub struct PrivateKey(pub [u8; 32]);
/// A Nano account public key
pub struct PublicKey(pub [u8; 32]);
/// A Nano account address
pub struct Address(pub [u8; 60]);
/// A Nano address pattern
///
/// An address pattern is a 60 characters string matching the
/// `^[13*][13-9a-km-uw-z*]{59}$` regex.
///
/// # Example
///
/// - `*mroger*****************************************************`
///
pub struct AddressPattern(pub [u8; 60]);

impl Seed {
    /// Derive a Nano private key
    pub fn derive_private_key(&self, index: u32) -> PrivateKey {
        let index_bytes = index.to_big_endian();

        let mut buffer = [0u8; 32];
        let mut hasher = Blake2b::new(32);
        hasher.input(&self.0);
        hasher.input(&index_bytes);
        hasher.result(&mut buffer);

        PrivateKey(buffer)
    }
}

impl PrivateKey {
    /// Derive a Nano public key
    pub fn derive_public_key(&self) -> PublicKey {
        let public_key = custom_ed25519::keypair(&self.0);

        PublicKey(public_key)
    }
}

impl PublicKey {
    /// Compute the public key checksum
    pub fn compute_checksum(&self) -> [u8; 5] {
        let mut checksum = [0u8; 5];
        let mut hasher = Blake2b::new(5);
        hasher.input(&self.0);
        hasher.result(&mut checksum);
        checksum.reverse();

        checksum
    }

    /// Encode an account address
    pub fn encode_address(&self) -> Address {
        let mut address = [0u8; 60];
        base32::encode(&self.0, &mut address[..52]);
        base32::encode(&self.compute_checksum(), &mut address[52..]);

        Address(address)
    }
}

impl Address {
    // Decode an account address
    pub fn decode(&self) -> Result<PublicKey, &'static str> {
        let public_key_part = &self.0[..52];
        let checksum_part = &self.0[52..];

        let mut public_key_buffer = [0u8; 33]; // the decode fn will use 33 bytes but return a slice of 32
        let public_key = PublicKey(*array_ref!(
            base32::decode(public_key_part, &mut public_key_buffer),
            0,
            32
        ));

        let mut checksum_buffer = [0u8; 6]; // the decode fn will use 6 bytes but return a slice of 5
        let checksum = *array_ref!(base32::decode(checksum_part, &mut checksum_buffer), 0, 5);
        let valid_checksum = public_key.compute_checksum();

        let is_checksum_valid = checksum
            .iter()
            .zip(valid_checksum.iter())
            .all(|(a, b)| a == b);

        if is_checksum_valid {
            Ok(public_key)
        } else {
            Err("The checksum is invalid")
        }
    }

    /// Match the address against a pattern
    pub fn match_pattern(&self, pattern: &AddressPattern) -> bool {
        pattern
            .0
            .iter()
            .enumerate()
            .all(|(i, &c)| c == b'*' || c == self.0[i])
    }
}