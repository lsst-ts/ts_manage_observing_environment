//! Manage observing environment for the Vera Rubin Observatory Control System.
//!
//! This package provides facility to manage the observing environment for the
//! Vera Rubin Observatory Control System. The observing environment consists
//! of a series of packages that are used by the control system in general for
//! regular operations. They are mainly designed to support the ScriptQueue
//! and users of the nublado Jupyter notebook environment.
//!
//! By managing these packages in a unified way, we can guarantee that users
//! and control system are using a uniform set of package versions and that we
//! are also able to quickly push fixes, patches and new features on the fly,
//! without the need to restart components or user environments.
//!
//! The main package that provides the core functionality for the observing
//! environment is `manage_obs_env`. This cli can setup the environment, cloning
//! all the repositories and creating a setup file for the environment. It also
//! allow users to checkout specific versions of each package, rollback to a
//! sanctioned set of versions and other activities. Each activity is also logged
//! into the Engineering Facility Database, which allows us to track which versions
//! of the packages are in use at every time, and also who performed the action.
//!
//! A second cli available in this package is `obs_env_sidecar`. This application,
//! when executed, will setup the environment and monitor any activity performed in
//! the observing environment, replicating the activity in its own environment. The
//! idea is to reduce the reliance on NFS to be able to use the observing environment.
//!
#![doc = include_str!("../CHANGELOG.md")]

#[macro_use]
extern crate serde_derive;
pub mod error;
pub mod manage_obs_env;
pub mod obs_env_sidecar;
pub mod observing_environment;
pub mod repos;
pub mod sasquatch;
