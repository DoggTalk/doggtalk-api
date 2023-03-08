mod error_code;
mod result;

pub use error_code::ApiErrorCode;
pub use result::*;

pub fn init() {
    error_code::init();
}
