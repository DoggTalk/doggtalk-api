use std::collections::HashMap;

use super::super::base::*;

#[allow(dead_code)]
pub enum ApiErrorCode {
    Success = 0,
    // inner error
    InvalidDatabase = 1001,
    // public error
    InvalidParams = 2001,
    InvalidSign = 2002,
    InvalidToken = 2003,
    AccountNotFound = 3001,
    AccountOrPasswordFailed = 3002,
    NoPermission = 3003,
    AccountNotActived = 3004,
    AppNotFound = 4001,
    TopicNotFound = 5001,
    ReplyNotFound = 5101,
    Unexpected = 9999,
}

static ERROR_CODE_MAP: Lazy<HashMap<i32, String>> =
    Lazy::new(|| load_json(&mut ["res", "error_code.json"]));

pub fn init() {
    Lazy::force(&ERROR_CODE_MAP);
}

pub fn render_error(code: ApiErrorCode, error: &str) -> (i32, String) {
    let code = code as i32;
    let text = match ERROR_CODE_MAP.get(&code) {
        Some(text) => {
            let mut out = String::from(text);
            if out.ends_with(":") {
                out.push(' ');
                out.push_str(error);
            }
            out
        }
        _ => String::from(error),
    };
    return (code, text);
}
