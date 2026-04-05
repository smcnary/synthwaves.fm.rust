use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use anyhow::Context;
use chrono::{Duration, Utc};
use domain::auth::AuthError;
use hmac::{Hmac, Mac};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use md5::{Digest, Md5};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    pub user_id: Uuid,
    pub api_key_id: Uuid,
    pub exp: usize,
}

pub fn issue_jwt(user_id: Uuid, api_key_id: Uuid, secret: &str) -> anyhow::Result<String> {
    let exp = (Utc::now() + Duration::hours(1)).timestamp() as usize;
    let claims = JwtClaims {
        user_id,
        api_key_id,
        exp,
    };
    Ok(encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?)
}

pub fn decode_jwt(token: &str, secret: &str) -> Result<JwtClaims, AuthError> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;
    decode::<JwtClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map(|data| data.claims)
    .map_err(|_| AuthError::InvalidCredentials)
}

pub fn secure_compare(lhs: &str, rhs: &str) -> anyhow::Result<bool> {
    let mut left_mac =
        HmacSha256::new_from_slice(b"synthwaves-secure-compare").context("invalid hmac key")?;
    left_mac.update(lhs.as_bytes());
    let left = left_mac.finalize().into_bytes();

    let mut right_mac =
        HmacSha256::new_from_slice(b"synthwaves-secure-compare").context("invalid hmac key")?;
    right_mac.update(rhs.as_bytes());
    let right = right_mac.finalize().into_bytes();

    Ok(left.eq(&right))
}

pub fn validate_subsonic_token(stored_password: &str, salt: &str, provided_t: &str) -> bool {
    let mut hasher = Md5::new();
    hasher.update(stored_password.as_bytes());
    hasher.update(salt.as_bytes());
    let expected = format!("{:x}", hasher.finalize());
    expected.eq_ignore_ascii_case(provided_t)
}

pub fn decode_subsonic_password(input: &str) -> String {
    if !input.starts_with("enc:") {
        return input.to_string();
    }
    let hex = input.trim_start_matches("enc:");
    hex.as_bytes()
        .chunks(2)
        .filter_map(|chunk| std::str::from_utf8(chunk).ok())
        .filter_map(|part| u8::from_str_radix(part, 16).ok())
        .map(char::from)
        .collect()
}

pub fn hash_password(plain: &str) -> anyhow::Result<String> {
    let salt = SaltString::encode_b64(Uuid::new_v4().as_bytes())
        .map_err(|err| anyhow::anyhow!("failed to encode salt bytes: {err}"))?;
    let hash = Argon2::default()
        .hash_password(plain.as_bytes(), &salt)
        .map_err(|err| anyhow::anyhow!("failed to hash password: {err}"))?
        .to_string();
    Ok(hash)
}

pub fn verify_password(plain: &str, password_hash: &str) -> bool {
    let parsed = PasswordHash::new(password_hash);
    let Ok(parsed) = parsed else {
        return false;
    };
    Argon2::default()
        .verify_password(plain.as_bytes(), &parsed)
        .is_ok()
}

#[cfg(test)]
mod tests {
    use super::{hash_password, verify_password};

    #[test]
    fn hash_and_verify_password_round_trip() -> anyhow::Result<()> {
        let hash = hash_password("synthwave-forever")?;
        assert!(verify_password("synthwave-forever", &hash));
        Ok(())
    }

    #[test]
    fn verify_password_rejects_wrong_secret() -> anyhow::Result<()> {
        let hash = hash_password("correct-horse-battery-staple")?;
        assert!(!verify_password("wrong-password", &hash));
        Ok(())
    }
}
