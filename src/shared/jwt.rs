use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};

use super::base::*;
use super::web::*;

static JWT_KEYS: Lazy<Keys> = Lazy::new(|| {
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    Keys::new(secret.as_bytes())
});
const JWT_TTL: i64 = 30 * 24 * 3600;
const JWT_ALGORITHM: Algorithm = Algorithm::HS384;

struct Keys {
    encoding: EncodingKey,
    decoding: DecodingKey,
}

impl Keys {
    fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Claims {
    tc: String,
    v: String,
    ts: i64,
}

pub fn init() {
    Lazy::force(&JWT_KEYS);
}

pub fn jwt_build(tc: &str, v: String) -> Result<String, ApiError> {
    let claims = Claims {
        tc: String::from(tc),
        v,
        ts: timestamp(),
    };

    let header = Header::new(JWT_ALGORITHM);
    return encode(&header, &claims, &JWT_KEYS.encoding)
        .map_err(|_| api_error2(ApiErrorCode::Unexpected, "token creation"));
}

pub fn jwt_parse(tc: &str, token: &str) -> Result<String, ApiError> {
    let mut validation = Validation::new(JWT_ALGORITHM);
    validation.required_spec_claims.clear();

    let res = decode::<Claims>(token, &JWT_KEYS.decoding, &validation)
        .map_err(|_| api_error2(ApiErrorCode::InvalidToken, "decode"))?;

    if res.claims.tc != tc || res.claims.ts + JWT_TTL <= timestamp() {
        return Err(api_error2(ApiErrorCode::InvalidToken, "expired"));
    } else {
        return Ok(res.claims.v);
    }
}
