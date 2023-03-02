use chrono::prelude::Utc;

pub fn timestamp() -> u64 {
    return Utc::now().timestamp() as u64;
}
