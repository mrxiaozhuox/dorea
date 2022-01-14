use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};

use super::db::ServiceAccountInfo;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    iat: usize,
    exp: usize,
    sub: String,
    pub account: ServiceAccountInfo,
}

pub struct Secret {
    pub(crate) token: String,
}

impl Secret {
    // Secret.apply
    // this function will apply a JWT Token for the Dorea-Service
    pub fn apply(&self, account: ServiceAccountInfo, expire: usize) -> crate::Result<String> {
        let now = chrono::Local::now().timestamp() as usize;

        let the_claims = Claims {
            iat: now,
            exp: now + expire,
            sub: "DOREA@SERVICE".to_string(),
            account,
        };

        let token = encode(
            &Header::default(),
            &the_claims,
            &EncodingKey::from_secret(self.token.as_ref()),
        )?;

        Ok(token)
    }

    // Secret.validation
    // this function will check the token
    pub fn validation(&self, token: String) -> crate::Result<TokenData<Claims>> {
        let token = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(self.token.as_ref()),
            &Validation::default(),
        )?;

        Ok(token)
    }
}

// 验证参数结构体
#[derive(Deserialize)]
pub struct SecretForm {
    pub account: String,
    pub password: String,
}
