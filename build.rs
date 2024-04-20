use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    fs::write(out_dir.join("link.x"), include_bytes!("link.x")).unwrap();

    println!("cargo:rustc-link-search={}", out_dir.display());
    println!("cargo:rerun-if-changed=link.x");
    println!("cargo:rerun-if-changed=src/boot/boot.s");
    println!("cargo:rerun-if-changed=build.rs");
}
