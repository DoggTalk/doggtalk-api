pub use serde::{Deserialize, Serialize};
pub use validator::Validate;

mod error_code;
mod result;
mod validate;
mod wrapper;

pub use error_code::ApiErrorCode;
pub use result::*;
pub use validate::*;
pub use wrapper::*;

pub fn init() {
    error_code::init();
}
