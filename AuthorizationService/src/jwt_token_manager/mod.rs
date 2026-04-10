use hmac::digest::{InvalidLength, Output};
use hmac::{Hmac, KeyInit, Mac};
use jsonwebtoken::{EncodingKey, Header};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    uuid: String,
}

#[derive(Clone)]
pub(crate) struct JWTTokenManager {
    encoding_key: Output<Sha256>,
}

impl JWTTokenManager {
    pub(crate) fn init(key: &str) -> Result<Self, InvalidLength> {
        let mac = HmacSha256::new_from_slice(key.as_bytes())?;
        let res = mac.finalize().into_bytes();
        Ok(Self { encoding_key: res })
    }

    pub(crate) fn get_jwt_token(&self, uuid: String) -> jsonwebtoken::errors::Result<String> {
        let claim = Claims { uuid };
        jsonwebtoken::encode(
            &Header::default(),
            &claim,
            &EncodingKey::from_secret(self.encoding_key.as_slice()),
        )
    }
}
