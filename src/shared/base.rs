use chrono::prelude::Utc;

pub fn timestamp() -> i64 {
    return Utc::now().timestamp();
}
