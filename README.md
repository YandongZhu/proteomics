# Proteomic Workflow Scaffold (experimental stage)

A minimal runnable scaffold: one workflow node, `LoadData`, corresponding to
a single Docker container.

## Directory layout

```
proto/workflow.proto       # Control/metadata protocol (DataReference / LoadDataConfig / LoadDataResult)
crates/workflow-core/       # WorkflowExecutor trait + LocalDockerExecutor implementation
src-tauri/                  # Tauri v2 app, exposes #[tauri::command] handle_load_data
dist/                       # Minimal frontend (static HTML calling handle_load_data)
docker/load_data/           # LoadData node image (Python algorithm + Dockerfile)
sample_data/input/          # Simulated raw mass spectrometry data (one spectrum record per line)
```

## Architecture highlights

- **Protocol layer**: nodes don't pass raw absolute file paths to each
  other; instead they pass a `DataReference` (`storage_type` + `uri`).
  Currently only `LOCAL_FILE` is used, with `S3_OBJECT` / `K8S_PVC` reserved
  so the message structure stays the same when switching storage backends.
- **Execution layer**: the `WorkflowExecutor` trait defines the node
  execution interface; `LocalDockerExecutor` is the single-machine
  implementation (`docker run --rm -v ...`). When migrating to K8s in the
  future, add a `K8sJobExecutor` (submits a Job + mounts a PVC) — the
  `handle_load_data` command needs no changes.
- **Data exchange**: host directories are bind-mounted via `-v` into
  `/data/input` (read-only) and `/data/output` inside the container. The
  algorithm serializes its structured result to `result.pb` in
  `/data/output`, which `LocalDockerExecutor` reads and decodes into
  `LoadDataResult`.

## Build / Run

1. Build the LoadData image (run from the repo root, requires Docker):

   ```sh
   docker build -f docker/load_data/Dockerfile -t proteomic/load-data:latest .
   ```

2. Run Tauri in dev mode (requires the Tauri CLI; `dist/` is already plain
   static files and is used directly as `frontendDist`):

   ```sh
   cargo tauri dev
   ```

3. In the opened window, click "Run LoadData". This runs the container with
   `sample_data/input` as the input directory and `sample_data/output` as
   the output directory, and displays the parsed `LoadDataResult` (JSON) on
   the page.

## Verifying workflow-core standalone

```sh
cargo check -p workflow-core
cargo check -p proteomic-app
```
