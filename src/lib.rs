//! # The SQRT library
//! The Scrypto Quick Rtm Testing library is a tool that enables its users to easily generate and use Radix Transaction Manifests to test a Scrypto package.
//!
//! # Examples
//! A wide variety of usage examples is available in the [test](tests) directory.
//!
//! # TODO for version 1.0
//! - [ ] Deal with return of blueprint methods
//! - [ ] Allow multiple arguments return when instantiating a function
//! - [ ] Allow multiple possible instantiation
//! - [ ] Deal with blueprints state
//! - [ ] Deal with returns and automatically check how things should have evolved
//! - [ ] Automatic implementation of method trait

extern crate core;
mod account;
pub mod blueprint;
mod component;
mod instructions;
mod manifest;
pub mod method;
pub mod package;
mod resource_manager;
pub mod test_environment;
mod transfer;
mod utils;
pub mod error;
mod manifest_call;
