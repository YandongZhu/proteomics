use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use async_trait::async_trait;
use prost::Message;
use tokio::process::Command;

use crate::executor::WorkflowExecutor;
use crate::pb::{DataReference, LoadDataConfig, LoadDataResult, NodeStatus, StorageType};

/// Executor that runs node containers locally via `docker run`.
///
/// This is the single-machine implementation of [`WorkflowExecutor`]: each
/// node = one disposable container (`docker run --rm ...`). Input/output
/// data is passed via `-v` bind mounts between host directories and
/// container directories (`/data/input`, `/data/output`), while control
/// parameters and structured results are passed via Protobuf files.
///
/// When migrating to K8s in the future, a new `K8sJobExecutor` can be added
/// that replaces `-v host_dir:/data/...` with a PVC + volumeMount and
/// replaces `docker run` with creating/waiting on a `Job`, while the
/// [`WorkflowExecutor`] interface exposed to callers stays the same.
pub struct LocalDockerExecutor {
    /// Image name for the LoadData node, e.g. `proteomic/load-data:latest`.
    image: String,
}

impl LocalDockerExecutor {
    pub fn new(image: impl Into<String>) -> Self {
        Self {
            image: image.into(),
        }
    }

    /// Resolve a [`DataReference`] into a local host directory path.
    ///
    /// Currently only `LOCAL_FILE` is supported; other storage types are
    /// reserved for future K8s/object-storage executors.
    fn local_dir_from_ref(data_ref: &DataReference, field_name: &str) -> Result<PathBuf> {
        if data_ref.storage_type != StorageType::LocalFile as i32 {
            bail!(
                "LocalDockerExecutor only supports StorageType::LOCAL_FILE, but `{}` has storage_type = {}",
                field_name,
                data_ref.storage_type
            );
        }
        if data_ref.uri.is_empty() {
            bail!("`{}` has an empty uri", field_name);
        }
        Ok(PathBuf::from(&data_ref.uri))
    }
}

#[async_trait]
impl WorkflowExecutor for LocalDockerExecutor {
    async fn run_load_data(&self, config: &LoadDataConfig) -> Result<LoadDataResult> {
        let input_ref = config
            .input_data
            .as_ref()
            .ok_or_else(|| anyhow!("LoadDataConfig.input_data is not set"))?;
        let output_ref = config
            .output_directory
            .as_ref()
            .ok_or_else(|| anyhow!("LoadDataConfig.output_directory is not set"))?;

        let input_dir = Self::local_dir_from_ref(input_ref, "input_data")?;
        let output_dir = Self::local_dir_from_ref(output_ref, "output_directory")?;

        if !input_dir.is_dir() {
            bail!("Input directory does not exist or is not a directory: {}", input_dir.display());
        }
        tokio::fs::create_dir_all(&output_dir)
            .await
            .with_context(|| format!("Failed to create output directory: {}", output_dir.display()))?;

        // Docker bind mounts require absolute host paths, so canonicalize
        // any relative paths (resolved against the process's working
        // directory) before passing them to `docker run -v`.
        let input_dir = tokio::fs::canonicalize(&input_dir)
            .await
            .with_context(|| format!("Failed to resolve input directory: {}", input_dir.display()))?;
        let output_dir = tokio::fs::canonicalize(&output_dir)
            .await
            .with_context(|| format!("Failed to resolve output directory: {}", output_dir.display()))?;

        // Fixed mount points inside the container: the node algorithm only
        // needs to know about /data/input and /data/output, and is unaware
        // of the real host paths, which makes it easy to swap in PVC mount
        // points when running on K8s in the future.
        const CONTAINER_INPUT_DIR: &str = "/data/input";
        const CONTAINER_OUTPUT_DIR: &str = "/data/output";

        let status = Command::new("docker")
            .arg("run")
            .arg("--rm")
            .arg("-v")
            .arg(format!("{}:{}:ro", input_dir.display(), CONTAINER_INPUT_DIR))
            .arg("-v")
            .arg(format!("{}:{}", output_dir.display(), CONTAINER_OUTPUT_DIR))
            .arg("-e")
            .arg(format!(
                "MIN_INTENSITY_THRESHOLD={}",
                config.min_intensity_threshold
            ))
            .arg("-e")
            .arg(format!(
                "ALLOWED_FILE_EXTENSIONS={}",
                config.allowed_file_extensions.join(",")
            ))
            .arg(&self.image)
            .status()
            .await
            .with_context(|| format!("Failed to start `docker run {}`", self.image))?;

        if !status.success() {
            bail!("Container {} failed, exit code: {:?}", self.image, status.code());
        }

        Self::read_result(&output_dir).await
    }
}

impl LocalDockerExecutor {
    /// After the container exits, read the algorithm's `result.pb` from the
    /// output directory and decode it into a [`LoadDataResult`].
    async fn read_result(output_dir: &Path) -> Result<LoadDataResult> {
        let result_path = output_dir.join("result.pb");
        let bytes = tokio::fs::read(&result_path)
            .await
            .with_context(|| format!("Failed to read algorithm output: {}", result_path.display()))?;

        let mut result = LoadDataResult::decode(bytes.as_slice())
            .with_context(|| format!("Failed to parse LoadDataResult from {}", result_path.display()))?;

        // If the algorithm script did not populate output_data, fill it in
        // with a reference to result.pb.
        if result.output_data.is_none() {
            result.output_data = Some(DataReference {
                storage_type: StorageType::LocalFile as i32,
                uri: result_path.to_string_lossy().into_owned(),
                metadata: Default::default(),
            });
        }

        if result.status == NodeStatus::Unspecified as i32 {
            result.status = NodeStatus::Success as i32;
        }

        Ok(result)
    }
}
