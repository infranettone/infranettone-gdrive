// `google_drive3::Error` is ~160 bytes and is carried by nearly every error
// enum here, so this lint fires ~60 times. Boxing it would change every public
// error type and construction site to save a few stack bytes on paths that
// only run once a request has already failed.
#![allow(clippy::result_large_err)]

pub mod about;
pub mod account;
pub mod app_config;
pub mod common;
pub mod drives;
pub mod files;
pub mod hub;
pub mod permissions;
pub mod version;
