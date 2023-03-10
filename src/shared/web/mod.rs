mod error_code;
mod result;
mod wrapper;

pub use error_code::ApiErrorCode;
pub use result::*;
pub use wrapper::*;

pub fn init() {
    error_code::init();
}
