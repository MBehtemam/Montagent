//! The font-vendoring rules: the three-bucket licence gate, the blocklist, and the
//! content hash the attestation carries (ADR-0057).
//!
//! Two verbs and one check read this module. `fonts list` asks [`licence::bucket`] so an
//! author *"sees a doomed vendor attempt before making it"*; `fonts vendor` asks it
//! **before any bytes are copied**; and `validate`'s attestation check asks [`sha256_hex`]
//! so the recorded hash and the file on disk are compared by the one function that wrote
//! the hash in the first place.
//!
//! What is here is rules — which names refuse, which licence texts are recognised, what a
//! hash is — and rules live in the core (ADR-0011). Reading the font file's own account of
//! itself is `montagent_text::names`'s.

pub mod licence;

use sha2::{Digest, Sha256};

/// The lowercase hex SHA-256 of some bytes — the `sha256` a `fontVendor` entry records.
pub fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_of_the_empty_input_is_the_published_constant() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn sha256_of_abc_is_the_published_constant() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
