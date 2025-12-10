#[cfg(feature = "grpc")]
fn compile_protos() {
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    tonic_prost_build::configure()
        .file_descriptor_set_path(out_dir.join("terms_of_use_descriptor.bin"))
        .compile_protos(&["proto/service.proto"], &["proto"])
        .unwrap();
}

fn main() {
    #[cfg(feature = "grpc")]
    compile_protos();
}
