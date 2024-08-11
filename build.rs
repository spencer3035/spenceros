use std::{
    path::{Path, PathBuf},
    process::Command,
};

fn build_entry(out_dir: &Path) -> PathBuf {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());

    let local_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("bootloader")
        .join("entry");

    println!("cargo:rerun-if-changed={}", local_path.display());

    let mut cmd = Command::new(cargo);
    cmd.arg("install")
        .arg("--path")
        .arg(local_path)
        .arg("--locked")
        .arg("--target")
        .arg("tuples/i386-code16-boot-sector.json")
        .arg("-Zbuild-std=core")
        .arg("-Zbuild-std-features=compiler-builtins-mem")
        .arg("--root")
        .arg(out_dir)
        .env_remove("RUSTFLAGS")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .env_remove("RUSTC_WORKSPACE_WRAPPER");

    let status = cmd.status().expect("Failed to run cargo command");
    assert!(status.success(), "Failed running cargo command");
    let elf_file = out_dir.join("bin").join("entry");

    assert!(
        elf_file.exists(),
        "Didn't find bootloader file {}",
        elf_file.display()
    );

    //objcopy -I elf32-i386 -O binary target/i386-code16-boot-sector/release/spenceros2 ${TARGET}
    //objcopy -I elf32-i386 -O binary target/i386-code16-boot-sector/release/spenceros2 ${TARGET}
    //
    let mut cmd = Command::new("objcopy");
    let mut bin_file = elf_file.clone();
    bin_file.set_extension("bin");

    cmd.arg("-I")
        .arg("elf32-i386")
        .arg("-O")
        .arg("binary")
        .arg(&elf_file)
        .arg(&bin_file);

    let status = cmd.status().expect("Failed to convert to elf");
    assert!(status.success(), "Got nonzero exit code");
    bin_file
}

fn main() {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    let file = build_entry(&out_dir);
    println!("cargo:rustc-env=BIOS_ENTRY={}", file.display());
}
