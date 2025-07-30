use std::time::{SystemTime, SystemTimeError};

use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};

pub(crate) fn current_timestamp() -> Result<u64, SystemTimeError> {
    Ok(SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_secs())
}

pub(crate) fn sha256_hex(message: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(message);
    format!("{:x}", hasher.finalize()).to_lowercase()
}

pub(crate) fn hmac256(key: &[u8], message: &str) -> Result<Vec<u8>, String> {
    let mut mac = Hmac::<Sha256>::new_from_slice(key)
        .map_err(|e| format!("use data key on sha256 fail:{e}"))?;
    mac.update(message.as_bytes());
    let signature = mac.finalize();
    Ok(signature.into_bytes().to_vec())
}
