fn main() {
    // Use the precompiled protoc shipped by protoc-bin-vendored so we don't
    // depend on a system protoc/cmake installation.
    std::env::set_var(
        "PROTOC",
        protoc_bin_vendored::protoc_bin_path().expect("failed to locate vendored protoc"),
    );

    let mut config = prost_build::Config::new();
    // Make the generated message types also support serde serialization,
    // so they can be passed directly as Tauri command arguments / return
    // values over IPC.
    config.type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]");
    config.type_attribute(".", "#[serde(rename_all = \"camelCase\")]");

    config
        .compile_protos(&["../../proto/workflow.proto"], &["../../proto"])
        .expect("failed to compile workflow.proto");

    println!("cargo:rerun-if-changed=../../proto/workflow.proto");
}
