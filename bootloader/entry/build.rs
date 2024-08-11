use std::path::Path;

fn main() {
    let local_path = Path::new(env!("CARGO_MANIFEST_DIR"));
    let asm_path = local_path
        .join("bootloader")
        .join("entry")
        .join("src")
        .join("boot.s");
    println!("cargo:rerun-if-changed={}", asm_path.display());
    println!(
        "cargo:rustc-link-arg-bins=--script={}",
        local_path.join("link.ld").display()
    )
}
