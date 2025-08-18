use std::{env, path::PathBuf};

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    tonic_prost_build::configure()
        .file_descriptor_set_path(out_dir.join("smm_descriptor.bin"))
        .compile_protos(&["proto/smm/v1/smm.proto"], &["proto/smm"])
        .unwrap();
}
