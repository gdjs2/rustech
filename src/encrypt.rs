use std::num;

use ring::{digest, pbkdf2, rand::{SecureRandom, SystemRandom}};

const CREDENTIAL_LEN: usize = digest::SHA512_OUTPUT_LEN;
const N_ITER: u32 = 107_831;

pub fn encrypt(
    password: &str
) -> Result<([u8; CREDENTIAL_LEN], [u8; CREDENTIAL_LEN]), ring::error::Unspecified> {

    let rng = SystemRandom::new();
    let mut salt = [0u8; CREDENTIAL_LEN];
    rng.fill(&mut salt)?;
    let n_iter: num::NonZeroU32 = num::NonZeroU32::new(N_ITER).unwrap();

    let mut hash = [0u8; CREDENTIAL_LEN];
    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA512, 
        n_iter, 
        &salt, 
        password.as_bytes(), 
        &mut hash
    );

    Ok((salt, hash))
}

pub fn verify(
    password: &str,
    salt: [u8; CREDENTIAL_LEN],
    hash: [u8; CREDENTIAL_LEN]
) -> bool {
    let n_iter: num::NonZeroU32 = num::NonZeroU32::new(N_ITER).unwrap();
    let result = pbkdf2::verify(
        pbkdf2::PBKDF2_HMAC_SHA512, 
        n_iter, 
        &salt, 
        password.as_bytes(), 
        &hash);
    
    result.is_ok()
}

#[cfg(test)]
mod tests {
    use data_encoding;
    #[test]
    fn test_encrypt() {
        let password = "hello world, this is a password";
        let (salt, hash) = super::encrypt(&password).unwrap();
        println!("Salt: {}", data_encoding::HEXUPPER.encode(&salt));
        println!("Hash: {}", data_encoding::HEXUPPER.encode(&hash));
    }

    #[test]
    fn test_encrypt_verify() {
        let password = "This is a password!";
        let wrong_password = "This is not a password!";
        let (salt, hash) = super::encrypt(&password).unwrap();
        assert!(super::verify(password, salt, hash));
        assert!(!super::verify(wrong_password, salt, hash));
    }
}