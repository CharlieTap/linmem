extern crate cbindgen;

use std::env;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let output_header_path = Path::new(&out_dir).join("liblinmem.h");

    let config = cbindgen::Config {
        language: cbindgen::Language::C,
        pragma_once: true,
        ..Default::default()
    };

    cbindgen::generate_with_config(env::var("CARGO_MANIFEST_DIR").unwrap(), config)
        .expect("Unable to generate header file")
        .write_to_file(&output_header_path);

    println!("cargo:rerun-if-changed=src/ffi.rs");
    println!("Header file generated at: {}", output_header_path.display());
}
