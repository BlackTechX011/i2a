// Permission is hereby granted, free of charge, to any person obtaining a
// copy of this software and associated documentation files (the "Software"),
// to deal in the Software without restriction, including without limitation
// the rights to use, copy, modify, merge, publish, distribute, sublicense,
// and/or sell copies of the Software, and to permit persons to whom the
// Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS
// OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
// FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

use crate::error::Error;

use aes::cipher::generic_array::GenericArray;
use chacha20::{
    cipher::{StreamCipher, StreamCipherSeek},
    ChaCha20,
};
use chacha20poly1305::{
    aead::{Aead, AeadInPlace},
    consts::U12,
    ChaCha20Poly1305,
};

use alloc::vec::Vec;

/// Nonce.
///
/// Upper 4 bytes are zeroed out, maximum number for nonce is `u64::MAX - 1`
///
/// https://geti2p.net/spec/ntcp2#chacha20-poly1305
pub struct Nonce {
    /// Nonce.
    nonce: u64,
}

impl Nonce {
    /// Create new [`Nonce`], starting from `nonce`.
    pub fn new(nonce: u64) -> Self {
        Self { nonce }
    }

    /// Get next nonce.
    ///
    /// The first 4 bytes of the 12-bytes are not used.
    ///
    /// Maximum value for for `nonce` is 2^64 - 2.
    pub fn next(&mut self) -> Option<GenericArray<u8, U12>> {
        let nonce = {
            let nonce = self.nonce;
            self.nonce = self.nonce.checked_add(1)?;
            nonce
        };

        let mut array = [0u8; 12];
        array.as_mut_slice()[4..].copy_from_slice(nonce.to_le_bytes().as_slice());

        Some(GenericArray::from(array))
    }
}

/// Chacha20Poly1305 instance.
pub struct ChaChaPoly {
    /// Nonce.
    nonce: Nonce,

    /// Internal cipher.
    cipher: ChaCha20Poly1305,
}

impl ChaChaPoly {
    /// Create new [`ChachaPoly`] instance.
    pub fn new(key: &[u8]) -> Self {
        let key: [u8; 32] = key.try_into().expect("valid chacha key");
        let key = GenericArray::from(key);

        Self {
            cipher: {
                use chacha20poly1305::aead::KeyInit;
                ChaCha20Poly1305::new(&key)
            },
            nonce: Nonce::new(0u64),
        }
    }

    /// Create new [`ChachaPoly`] instance with a custom `nonce`.
    pub fn with_nonce(key: &[u8], nonce: u64) -> Self {
        let key: [u8; 32] = key.try_into().expect("valid chacha key");
        let key = GenericArray::from(key);

        Self {
            cipher: {
                use chacha20poly1305::aead::KeyInit;
                ChaCha20Poly1305::new(&key)
            },
            nonce: Nonce::new(nonce),
        }
    }

    /// Encrypt `plaintext` and return ciphertext on success.
    pub fn encrypt(&mut self, plaintext: &[u8]) -> crate::Result<Vec<u8>> {
        self.cipher
            .encrypt(&self.nonce.next().ok_or(Error::NonceOverflow)?, plaintext)
            .map_err(From::from)
    }

    /// Decrypto `ciphertext` and return plaintext on success.
    pub fn decrypt(&mut self, ciphertext: Vec<u8>) -> crate::Result<Vec<u8>> {
        self.cipher
            .decrypt(
                &self.nonce.next().ok_or(Error::NonceOverflow)?,
                &ciphertext[..],
            )
            .map_err(From::from)
    }

    /// Encrypt `plaintext` in place, passing in associated data for authentication.
    ///
    /// Return authentication tag on success.
    pub fn encrypt_with_ad(
        &mut self,
        associated_data: &[u8],
        plaintext: &mut [u8],
    ) -> crate::Result<Vec<u8>> {
        let tag = self.cipher.encrypt_in_place_detached(
            &self.nonce.next().ok_or(Error::NonceOverflow)?,
            associated_data,
            plaintext,
        )?;

        Ok(tag.as_slice().to_vec())
    }

    /// Encrypt `plaintext` in place, passing in associated data for authentication.
    pub fn encrypt_with_ad_new(
        &mut self,
        associated_data: &[u8],
        plaintext: &mut Vec<u8>,
    ) -> crate::Result<()> {
        self.cipher
            .encrypt_in_place(
                &self.nonce.next().ok_or(Error::NonceOverflow)?,
                associated_data,
                plaintext,
            )
            .map_err(From::from)
    }

    /// Decrypt `ciphertext` in place, passing in associated data for authentication.
    ///
    /// Return authentication tag on sucess.
    //
    // TODO: change `ciphertext` type to something more convenient
    pub fn decrypt_with_ad(
        &mut self,
        associated_data: &[u8],
        ciphertext: &mut Vec<u8>,
    ) -> crate::Result<()> {
        self.cipher
            .decrypt_in_place(
                &self.nonce.next().ok_or(Error::NonceOverflow)?,
                associated_data,
                ciphertext,
            )
            .map_err(From::from)
    }
}

/// ChaCha cipher.
pub struct ChaCha {
    /// Internal cipher.
    cipher: ChaCha20,
}

impl ChaCha {
    /// Create new [`ChaCha`] instance with a custom `nonce`.
    pub fn with_nonce(key: &[u8], nonce: u64) -> Self {
        let key: [u8; 32] = key.try_into().expect("valid chacha key");
        let key = GenericArray::from(key);
        let mut nonce = Nonce::new(nonce);
        let next_nonce = nonce.next().expect("to succeed");

        Self {
            cipher: {
                use chacha20::cipher::KeyIvInit;

                let mut cipher = ChaCha20::new(&key, &next_nonce);
                cipher.seek(64);
                cipher
            },
        }
    }

    /// Create new [`ChaCha`] instance with a custom IV.
    pub fn with_iv(key: [u8; 32], iv: [u8; 12]) -> Self {
        let key = GenericArray::from(key);
        let iv = GenericArray::from(iv);

        Self {
            cipher: {
                use chacha20::cipher::KeyIvInit;

                let mut cipher = ChaCha20::new(&key, &iv);
                cipher.seek(64);
                cipher
            },
        }
    }

    /// Encrypt `plaintext` in place.
    pub fn encrypt_ref(&mut self, plaintext: &mut [u8]) {
        self.cipher.apply_keystream(plaintext);
    }

    /// Dencrypt `ciphertext` in place.
    pub fn decrypt_ref(&mut self, ciphertext: &mut [u8]) {
        self.cipher.apply_keystream(ciphertext)
    }

    /// Decrypt constant-size `ciphertext` and return it after encryption.
    pub fn decrypt<const SIZE: usize>(&mut self, mut ciphertext: [u8; SIZE]) -> [u8; SIZE] {
        self.cipher.apply_keystream(&mut ciphertext);
        ciphertext
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_nonce() {
        let mut nonce = Nonce::new(0);

        // verify the first nonce is zero
        assert_eq!(nonce.next(), Some(GenericArray::from([0u8; 12])));

        // verify the first nonce is zero
        let mut array = [0u8; 12];
        array[4] = 1;

        assert_eq!(nonce.next(), Some(GenericArray::from(array)));
    }

    #[test]
    fn nonce_exhausted() {
        let mut nonce = Nonce::new(u64::MAX - 1);

        let mut array = [0u8; 12];
        array[5..].fill(0xff);
        array[4] = 0xfe;

        assert_eq!(nonce.next(), Some(GenericArray::from(array)));
        assert_eq!(nonce.next(), None);
    }

    #[test]
    fn test_chacha20_and_chacha20poly1305() {
        let key = [0xaa; 32];
        let mut plaintext = [0xbb; 64];

        let ciphertext = ChaChaPoly::with_nonce(&key, 1337).encrypt(&plaintext).unwrap();

        ChaCha::with_nonce(&key, 1337u64).encrypt_ref(&mut plaintext);

        assert_eq!(ciphertext[..ciphertext.len() - 16], plaintext);
    }
}
