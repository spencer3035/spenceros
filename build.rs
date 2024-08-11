use std::{
    path::{Path, PathBuf},
    process::Command,
};

/// Builds a binary artifact from a given repo
fn build_artifact(local_path: &Path, out_dir: &Path) -> PathBuf {
    println!("cargo:rerun-if-changed={}", local_path.display());
    elf_to_bin(&build_elf(local_path, out_dir))
}

/// Converts cargo ELF artifacts to raw binary
fn elf_to_bin(elf_file: &Path) -> PathBuf {
    assert!(
        elf_file.exists(),
        "Didn't find bootloader file {}",
        elf_file.display()
    );

    //objcopy -I elf32-i386 -O binary file.elf file.bin
    let mut cmd = Command::new("objcopy");
    let mut bin_file = elf_file.to_owned();
    bin_file.set_extension("bin");
    cmd.arg("-I")
        .arg("elf32-i386")
        .arg("-O")
        .arg("binary")
        .arg(elf_file)
        .arg(&bin_file);

    let status = cmd.status().expect("Failed to convert to elf");
    assert!(status.success(), "Got nonzero exit code");
    bin_file
}

/// Builds a 16 bit elf file from a cargo package
fn build_elf(local_path: &Path, out_dir: &Path) -> PathBuf {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let mut cmd = Command::new(cargo);
    cmd.arg("install")
        .arg("--path")
        .arg(local_path)
        .arg("--locked")
        .arg("--target")
        .arg("tuples/i386-bit16.json")
        .arg("-Zbuild-std=core")
        .arg("-Zbuild-std-features=compiler-builtins-mem")
        .arg("--root")
        .arg(out_dir)
        .env_remove("RUSTFLAGS")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .env_remove("RUSTC_WORKSPACE_WRAPPER");

    let status = cmd.status().expect("Failed to run cargo command");
    assert!(status.success(), "Failed running cargo command");
    out_dir.join("bin").join(local_path.file_name().unwrap())
}

fn build_entry(out_dir: &Path) -> PathBuf {
    // Build ./bootloader/entry/
    let local_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("bootloader")
        .join("entry");
    build_artifact(&local_path, out_dir)
}

fn build_protected(out_dir: &Path) -> PathBuf {
    // Build ./bootloader/real_mode/
    let local_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("bootloader")
        .join("real_mode");
    build_artifact(&local_path, out_dir)
}

fn main() {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    let file = build_entry(&out_dir);
    println!("cargo:rustc-env=BIOS_ENTRY={}", file.display());
    let file = build_protected(&out_dir);
    println!("cargo:rustc-env=BIOS_PROTECTED={}", file.display());
}
