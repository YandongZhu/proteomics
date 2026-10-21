//! Core workflow abstractions and the local Docker executor.
//!
//! Design goal: a node's "control/metadata" is passed between systems via
//! Protobuf (the [`pb`] module), and a node's "execution" is abstracted via
//! the [`WorkflowExecutor`] trait. Currently this is implemented by
//! [`LocalDockerExecutor`] (local `docker run`); when migrating to K8s in
//! the future, only a new `K8sJobExecutor` implementing the same trait needs
//! to be added — the upper layer (Tauri commands) requires no changes.

pub mod docker_executor;
pub mod executor;

/// Message types generated from `proto/workflow.proto`.
pub mod pb {
    include!(concat!(env!("OUT_DIR"), "/proteomic.workflow.v1.rs"));
}

pub use docker_executor::LocalDockerExecutor;
pub use executor::WorkflowExecutor;
