use serde::{Serialize, Deserialize};
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey, TokenData};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    iat: usize,
    exp: usize,
    sub: String,
    pub level: String,
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

    pub fn validation(&self, token: String) -> crate::Result<TokenData<Claims>> {
        let token = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(self.token.as_ref()),
            &Validation::default()
        )?;

        Ok(token)
    }

}

// 验证参数结构体
#[derive(Deserialize)]
pub struct SecretForm {
    pub password: String,
}