use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    // sherpa-onnx 库路径
    // 优先级: SHERPA_ONNX_DIR 环境变量 > 本地 deps/sherpa-onnx > /opt/sherpa-onnx
    let sherpa_dir = env::var("SHERPA_ONNX_DIR").unwrap_or_else(|_| {
        let local_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
            .parent()
            .unwrap()
            .join("deps/sherpa-onnx");
        if local_path.exists() {
            local_path.to_str().unwrap().to_string()
        } else {
            "/opt/sherpa-onnx".to_string()
        }
    });

    println!("cargo:warning=Using sherpa-onnx from: {}", sherpa_dir);
    println!("cargo:rustc-link-search=native={}/lib", sherpa_dir);
    println!("cargo:rustc-link-lib=dylib=sherpa-onnx-c-api");
    println!("cargo:rustc-link-lib=dylib=onnxruntime");

    // 生成 sherpa-onnx Rust 绑定
    let bindings_path = PathBuf::from(env::var("OUT_DIR").unwrap())
        .join("sherpa_bindings.rs");

    if !bindings_path.exists() {
        let header_path = format!("{}/include/sherpa-onnx/c-api/c-api.h", sherpa_dir);

        println!("cargo:rerun-if-changed={}", header_path);

        let bindings = bindgen::Builder::default()
            .header(header_path)
            .allowlist_function("SherpaOnnx.*")
            .allowlist_type("SherpaOnnx.*")
            .generate()
            .expect("Failed to generate sherpa-onnx bindings");

        bindings
            .write_to_file(&bindings_path)
            .expect("Failed to write bindings");
    }

    // 生成 cbindgen C 头文件 (for fcitx5-vinput)
    // 临时禁用以解决条件编译导致的 cbindgen 解析问题
    // TODO: 修复 cbindgen 配置或代码结构
    /*
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let output_file = PathBuf::from(&crate_dir)
        .parent()
        .unwrap()
        .join("target/vinput_core.h");

    if let Ok(config) = cbindgen::Config::from_file("cbindgen.toml") {
        cbindgen::Builder::new()
            .with_crate(crate_dir)
            .with_config(config)
            .generate()
            .expect("Failed to generate C bindings")
            .write_to_file(output_file);
    }
    */
}
