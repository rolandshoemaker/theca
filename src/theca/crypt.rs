//  _   _                    
// | |_| |__   ___  ___ __ _ 
// | __| '_ \ / _ \/ __/ _` |
// | |_| | | |  __/ (_| (_| |
//  \__|_| |_|\___|\___\__,_|
//
// license under the MIT license <http://opensource.org/licenses/MIT>
//
// crypt.rs
//   defintions of the AES encryption, decryption, and key derivation
//   function required to read encrypted profiles.

use std::iter::{repeat};
use crypto::{symmetriccipher, buffer, aes, blockmodes};
use crypto::buffer::{ReadBuffer, WriteBuffer, BufferResult};
use crypto::pbkdf2::{pbkdf2};
use crypto::hmac::{Hmac};
use crypto::sha2::{Sha256};
use crypto::digest::{Digest};
use crypto::fortuna::{Fortuna};
use std::rand::{SeedableRng, Rng};

// ALL the encryption functions thx rust-crypto ^_^
pub fn encrypt(data: &[u8], key: &[u8]) -> Result<Vec<u8>, symmetriccipher::SymmetricCipherError> {
    let mut iv = [0u8; 16];
    let mut f: Fortuna = SeedableRng::from_seed(data);
    f.fill_bytes(&mut iv);

    let mut encryptor = aes::cbc_encryptor(
            aes::KeySize::KeySize256,
            key,
            &iv,
            blockmodes::PkcsPadding);

    let mut final_result = Vec::<u8>::new();
    final_result.push_all(&iv);
    let mut read_buffer = buffer::RefReadBuffer::new(data);
    let mut buffer = [0; 4096];
    let mut write_buffer = buffer::RefWriteBuffer::new(&mut buffer);

    loop {
        let result = try!(encryptor.encrypt(&mut read_buffer, &mut write_buffer, true));

        final_result.push_all(write_buffer.take_read_buffer().take_remaining());

        match result {
            BufferResult::BufferUnderflow => break,
            BufferResult::BufferOverflow => {}
        }
    }

    Ok(final_result)
}

pub fn decrypt(encrypted_data: &[u8], key: &[u8]) -> Result<Vec<u8>, symmetriccipher::SymmetricCipherError> {
    let iv = &encrypted_data[0..16];

    let mut decryptor = aes::cbc_decryptor(
            aes::KeySize::KeySize256,
            key,
            iv,
            blockmodes::PkcsPadding);

    let mut final_result = Vec::<u8>::new();
    let mut read_buffer = buffer::RefReadBuffer::new(&encrypted_data[16..]);
    let mut buffer = [0; 4096];
    let mut write_buffer = buffer::RefWriteBuffer::new(&mut buffer);

    loop {
        let result = try!(decryptor.decrypt(&mut read_buffer, &mut write_buffer, true));
        final_result.push_all(write_buffer.take_read_buffer().take_remaining());
        match result {
            BufferResult::BufferUnderflow => break,
            BufferResult::BufferOverflow => {}
        }
    }

    Ok(final_result)
}

pub fn password_to_key(p: &str) -> Vec<u8> {
    // yehh.... idk
    let mut salt_sha = Sha256::new();
    salt_sha.input(p.as_bytes());
    let salt = salt_sha.result_str();

    let mut mac = Hmac::new(Sha256::new(), p.as_bytes());
    let mut key: Vec<u8> = repeat(0).take(32).collect();

    pbkdf2(&mut mac, salt.as_bytes(), 2056, key.as_mut_slice());

    key
}