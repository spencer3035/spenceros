use std::{
    path::{Path, PathBuf},
    process::Command,
};

/// Converts cargo ELF artifacts to raw binary
fn elf_to_bin(elf_file: &Path, bits: &NBits) -> PathBuf {
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
        .arg(bits.get_elf_target())
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

impl NBits {
    fn get_elf_target(&self) -> String {
        match self {
            NBits::Bits16 => "elf32-i386".into(),
            NBits::Bits32 => "elf32-i386".into(),
            NBits::Bits64 => "elf64-x86-64".into(),
        }
    }
    fn from_stage_number(stage_number: usize) -> Self {
        match stage_number {
            0 => NBits::Bits16,
            1 => NBits::Bits16,
            2 => NBits::Bits32,
            3 => NBits::Bits64,
            _ => {
                panic!("NBits not defined for stage {stage_number}");
            }
        }
    }
}

/// Builds a `bits` bit elf file from a cargo package
fn build_elf(local_path: &Path, out_dir: &Path, bits: &NBits) -> PathBuf {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let mut cmd = Command::new(cargo);
    let target = match bits {
        NBits::Bits16 => "tuples/i386-bit16.json",
        NBits::Bits32 => "tuples/i386-bit32.json",
        NBits::Bits64 => "tuples/x86_64-unknown-kernel.json",
    };
    cmd.arg("install")
        .arg("--color")
        .arg("always")
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

fn build_stage(out_dir: &Path, stage_number: usize) -> PathBuf {
    let stage_string = format!("stage-{stage_number}");
    let nbits = NBits::from_stage_number(stage_number);
    // Build ./bootloader/stage-{stage_number}/
    let local_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("bootloader")
        .join(stage_string);
    println!("cargo:rerun-if-changed={}", local_path.display());
    elf_to_bin(&build_elf(&local_path, out_dir, &nbits), &nbits)
}

fn main() {
    // Build ./bootloader/common/
    let common_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("bootloader")
        .join("common");
    println!("cargo:rerun-if-changed={}", common_path.display());

    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    let num_stages = 4;
    let mut handles = Vec::new();
    for stage in 0..num_stages {
        let out_dir = out.clone();
        let h = std::thread::spawn(move || {
            let file = build_stage(&out_dir, stage);
            println!("cargo:rustc-env=BIOS_STAGE{stage}={}", file.display());
        });

        handles.push(h);
    }

    for h in handles.into_iter() {
        h.join().unwrap();
    }
}
