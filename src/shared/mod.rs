pub mod base;
pub mod data;
pub mod hash;
pub mod jwt;
pub mod model;
pub mod web;

pub fn init() {
    jwt::init();
    data::init();
}
