use serde::{Serialize, Deserialize};
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    iat: usize,
    exp: usize,
    sub: String,
    level: String,
}

pub struct Secret {
    pub(crate) token: String
}

impl Secret {
    pub fn apply(&self ,level: String ,expire: usize) -> crate::Result<String> {

        let now = chrono::Local::now().timestamp() as usize;

        let the_claims = Claims {
            iat: now,
            exp: now + expire,
            sub: "DOREA@SERVICE".to_string(),
            level
        };

        let token = encode(
            &Header::default(),
            &the_claims,
            &EncodingKey::from_secret(self.token.as_ref())
        )?;

        Ok(token)
    }
}

// 验证参数结构体
#[derive(Deserialize)]
pub struct SecretParams {
    pub password: String,
}