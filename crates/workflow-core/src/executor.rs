use crate::pb::{LoadDataConfig, LoadDataResult};
use anyhow::Result;
use async_trait::async_trait;

/// Execution abstraction for a workflow node.
///
/// Each node type (LoadData, Search, Quantify, ...) corresponds to a method
/// on this trait, with inputs/outputs using the message types defined in
/// `proto/workflow.proto`. This way, regardless of whether a node ultimately
/// runs in a local Docker container or a K8s Job, the caller (a Tauri
/// command) sees a consistent interface.
///
/// When adding a `K8sJobExecutor` in the future, only this trait needs to be
/// implemented:
/// - `run_load_data` would instead submit a K8s Job (mounting a PVC instead
///   of using `-v` for a local directory), poll the Job status, and then
///   read `result.pb` from the output PVC.
#[async_trait]
pub trait WorkflowExecutor: Send + Sync {
    /// Run the LoadData node: parse raw mass spectrometry data and produce a `LoadDataResult`.
    async fn run_load_data(&self, config: &LoadDataConfig) -> Result<LoadDataResult>;
}
