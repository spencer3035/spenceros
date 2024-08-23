use std::{
    path::{Path, PathBuf},
    process::Command,
};

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

enum NBits {
    Bits16,
    Bits32,
    Bits64,
}

/// Builds a 16 bit elf file from a cargo package
fn build_elf(local_path: &Path, out_dir: &Path, bits: NBits) -> PathBuf {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let mut cmd = Command::new(cargo);
    let target = match bits {
        NBits::Bits16 => "tuples/i386-bit16.json",
        NBits::Bits32 => "tuples/i386-bit32.json",
        NBits::Bits64 => "tuples/x86_64-unknown-kernel.json",
    };
    cmd.arg("install")
        .arg("--path")
        .arg(local_path)
        .arg("--locked")
        .arg("--target")
        .arg(target)
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

fn build_stage_0(out_dir: &Path) -> PathBuf {
    // Build ./bootloader/stage-0/
    let local_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("bootloader")
        .join("stage-0");
    println!("cargo:rerun-if-changed={}", local_path.display());
    elf_to_bin(&build_elf(&local_path, out_dir, NBits::Bits16))
}

fn build_stage_1(out_dir: &Path) -> PathBuf {
    // Build ./bootloader/stage-1/
    let local_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("bootloader")
        .join("stage-1");
    println!("cargo:rerun-if-changed={}", local_path.display());
    elf_to_bin(&build_elf(&local_path, out_dir, NBits::Bits16))
}

fn build_stage_2(out_dir: &Path) -> PathBuf {
    // Build ./bootloader/stage-2/
    let local_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("bootloader")
        .join("stage-2");
    println!("cargo:rerun-if-changed={}", local_path.display());
    elf_to_bin(&build_elf(&local_path, out_dir, NBits::Bits32))
}

fn build_stage_3(out_dir: &Path) -> PathBuf {
    // Build ./bootloader/stage-2/
    let local_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("bootloader")
        .join("stage-3");
    println!("cargo:rerun-if-changed={}", local_path.display());
    elf_to_bin(&build_elf(&local_path, out_dir, NBits::Bits64))
}

fn main() {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    // Build ./bootloader/common/
    let common_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("bootloader")
        .join("common");
    println!("cargo:rerun-if-changed={}", common_path.display());

    let file = build_stage_0(&out_dir);
    println!("cargo:rustc-env=BIOS_STAGE0={}", file.display());
    let file = build_stage_1(&out_dir);
    println!("cargo:rustc-env=BIOS_STAGE1={}", file.display());
    let file = build_stage_2(&out_dir);
    println!("cargo:rustc-env=BIOS_STAGE2={}", file.display());
    let file = build_stage_3(&out_dir);
    println!("cargo:rustc-env=BIOS_STAGE3={}", file.display());
}
