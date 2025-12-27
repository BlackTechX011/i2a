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

use aes::{
    cipher::{generic_array::GenericArray, BlockDecryptMut, BlockEncryptMut, IvState, KeyIvInit},
    Aes256,
};
use cbc::{Decryptor, Encryptor};

use alloc::vec::Vec;

/// AES operation kind.
enum AesKind {
    /// Encryptor.
    Encryptor {
        /// Inner AES object.
        aes: Encryptor<Aes256>,
    },

    /// Decryptor.
    Decryptor {
        /// Inner AES object.
        aes: Decryptor<Aes256>,
    },
}

/// AES encryptor/decryptor.
pub struct Aes {
    /// AES operation kind.
    kind: AesKind,
}

impl Aes {
    /// Create new [`Aes`] encryptor instance.
    pub fn new_encryptor(key: &[u8], iv: &[u8]) -> Self {
        let key: [u8; 32] = key.try_into().expect("valid aes key");
        let iv: [u8; 16] = iv.try_into().expect("valid aes iv");

        let aes = Encryptor::<Aes256>::new(&key.into(), &iv.into());

        Self {
            kind: AesKind::Encryptor { aes },
        }
    }

    /// Create new [`Aes`] decryptor instance.
    pub fn new_decryptor(key: &[u8], iv: &[u8]) -> Self {
        let key: [u8; 32] = key.try_into().expect("valid aes key");
        let iv: [u8; 16] = iv.try_into().expect("valid aes iv");

        let aes = Decryptor::<Aes256>::new(&key.into(), &iv.into());

        Self {
            kind: AesKind::Decryptor { aes },
        }
    }

    /// Encrypt `plaintext` using AES-CBC-256.
    ///
    /// Length of `plaintext` must be a multiple of 16
    pub fn encrypt<T: AsRef<[u8]>>(&mut self, plaintext: T) -> Vec<u8> {
        assert!(plaintext.as_ref().len() % 16 == 0, "invalid plaintext");

        let AesKind::Encryptor { aes } = &mut self.kind else {
            panic!("tried to call `encrypt()` for an aes decryptor");
        };

        let mut blocks = plaintext
            .as_ref()
            .chunks(16)
            .map(|chunk| {
                GenericArray::from(TryInto::<[u8; 16]>::try_into(chunk).expect("to succeed"))
            })
            .collect::<Vec<_>>();

        aes.encrypt_blocks_mut(&mut blocks);
        let _iv = aes.iv_state();

        blocks
            .into_iter()
            .flat_map(|block| block.into_iter().collect::<Vec<u8>>())
            .collect()
    }

    /// Dencrypt `ciphertext` using AES-CBC-256.
    ///
    /// Length of `ciphertext` must be a multiple of 16
    pub fn decrypt<T: AsRef<[u8]>>(&mut self, ciphertext: T) -> Vec<u8> {
        assert!(ciphertext.as_ref().len() % 16 == 0, "invalid ciphertext");

        let AesKind::Decryptor { aes } = &mut self.kind else {
            panic!("tried to call `decrypt()` for an aes encryptor");
        };

        let mut blocks = ciphertext
            .as_ref()
            .chunks(16)
            .map(|chunk| {
                GenericArray::from(TryInto::<[u8; 16]>::try_into(chunk).expect("to succeed"))
            })
            .collect::<Vec<_>>();

        aes.decrypt_blocks_mut(&mut blocks);
        blocks
            .into_iter()
            .flat_map(|block| block.into_iter().collect::<Vec<u8>>())
            .collect::<Vec<u8>>()
    }

    /// Get current IV.
    pub fn iv(&self) -> [u8; 16] {
        match &self.kind {
            AesKind::Encryptor { aes } => aes.iv_state().into(),
            AesKind::Decryptor { aes } => aes.iv_state().into(),
        }
    }
}
