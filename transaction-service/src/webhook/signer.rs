use hmac::{digest::InvalidLength, Hmac, Mac};
use sha2::Sha256;
use tracing::error;

use crate::constants::HMAC_SHA256_PREFIX;

type HmacSha256 = Hmac<Sha256>;

pub fn sign_hmac_sha256(secret: &str, body: &[u8]) -> Result<String, InvalidLength> {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).map_err(|e| {
        error!("HMAC init failed: {}", e);
        e
    })?;

    mac.update(body);

    let result = mac.finalize();
    let bytes = result.into_bytes();

    Ok(format!("{}{}", HMAC_SHA256_PREFIX, hex::encode(bytes)))
}
