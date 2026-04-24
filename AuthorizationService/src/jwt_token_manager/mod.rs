use hmac::digest::{InvalidLength, Output};
use hmac::{Hmac, KeyInit, Mac};
use jsonwebtoken::errors::ErrorKind;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH};

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    uuid: String,
    exp: u64,
}

#[derive(Clone)]
pub(crate) struct JWTTokenManager {
    encoding_key: Output<Sha256>,
}

pub(crate) enum JWTDecodingError {
    TokenExpired,
    InvalidToken,
    NoUUID,
}

impl JWTTokenManager {
    pub(crate) fn init(key: &str) -> Result<Self, InvalidLength> {
        let mac = HmacSha256::new_from_slice(key.as_bytes())?;
        let res = mac.finalize().into_bytes();
        Ok(Self { encoding_key: res })
    }

    pub(crate) fn get_jwt_token(&self, uuid: sqlx::types::Uuid) -> String {
        let exp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Should not fail")
            .as_secs()
            + 3600 * 1000;
        let claim = Claims {
            uuid: uuid.to_string(),
            exp,
        };
        jsonwebtoken::encode(
            &Header::default(),
            &claim,
            &EncodingKey::from_secret(self.encoding_key.as_slice()),
        )
        .expect("Should not fail")
    }

    pub(crate) fn get_uuid_from_token(
        &self,
        token: &[u8],
    ) -> Result<sqlx::types::Uuid, JWTDecodingError> {
        let decode_data: jsonwebtoken::errors::Result<TokenData<Claims>> = jsonwebtoken::decode(
            token,
            &DecodingKey::from_secret(self.encoding_key.as_slice()),
            &Validation::new(Algorithm::HS256),
        );
        match decode_data {
            Ok(data) => match sqlx::types::Uuid::try_parse(data.claims.uuid.as_str()) {
                Ok(uuid) => Ok(uuid),
                Err(_) => Err(JWTDecodingError::NoUUID),
            },
            Err(err) => match err.kind() {
                ErrorKind::ExpiredSignature => Err(JWTDecodingError::TokenExpired),
                _ => Err(JWTDecodingError::InvalidToken),
            },
        }
    }
}
