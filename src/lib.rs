//! Manage observing environment for the Vera Rubin Observatory Control System.
//!
#![doc = include_str!("../CHANGELOG.md")]

#[macro_use]
extern crate serde_derive;
pub mod error;
pub mod manage_obs_env;
pub mod observing_environment;
pub mod repos;
pub mod sasquatch;
