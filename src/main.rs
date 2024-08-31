use std::path::Path;

use common::{
    SECTORS_TO_READ, STAGE_0_SECTIONS, STAGE_1_SECTIONS, STAGE_2_SECTIONS, STAGE_3_SECTIONS,
};

const BOOT_0: &[u8] = include_bytes!(env!("BIOS_STAGE0"));
const BOOT_1: &[u8] = include_bytes!(env!("BIOS_STAGE1"));
const BOOT_2: &[u8] = include_bytes!(env!("BIOS_STAGE2"));
const BOOT_3: &[u8] = include_bytes!(env!("BIOS_STAGE3"));
const EXTRA_BYTES: [u8; 512] = [0; 512];

fn assert_sizes() {
    // First section needs to always be 512 bytes
    assert_eq!(BOOT_0.len(), 512, "boot entry point was not correct size");
    // Check section 1 is the correct size
    assert_eq!(
        BOOT_1.len(),
        512 * STAGE_1_SECTIONS,
        "boot stage 1 was not correct size (0x{:x} sections)",
        BOOT_1.len() / 0x200
    );
    // Check section 2 is the correct size
    assert_eq!(
        BOOT_2.len(),
        512 * STAGE_2_SECTIONS,
        "boot stage 2 was not correct size (0x{:x} sections)",
        BOOT_2.len() / 0x200
    );
    // Check section 2 is the correct size
    assert_eq!(
        BOOT_3.len(),
        512 * STAGE_3_SECTIONS,
        "boot stage 3 was not correct size (0x{:x} sections)",
        BOOT_3.len() / 0x200
    );

    // If this fails, need to read more sectors in stage 0 or 1
    let total_sectors = STAGE_0_SECTIONS + STAGE_1_SECTIONS + STAGE_2_SECTIONS + STAGE_3_SECTIONS;
    assert_eq!(
        total_sectors,
        SECTORS_TO_READ + 1,
        "Total sectors did not match expected"
    );
    assert!(
        total_sectors <= u8::MAX as usize,
        "too many sectors to read with one u8"
    );
}

fn main() {
    assert_sizes();
    // Put all sections together
    let disk_bytes: Vec<u8> = BOOT_0
        .iter()
        .chain(BOOT_1.iter())
        .chain(BOOT_2.iter())
        .chain(BOOT_3.iter())
        .chain(EXTRA_BYTES.iter())
        .cloned()
        .collect();

    // Write to file
    let disk_image_file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("disk.img");
    std::fs::write(&disk_image_file, &disk_bytes).unwrap();

    // Launch qemu
    let mut cmd = std::process::Command::new("qemu-system-x86_64");
    let extra_args = format!(
        "file={},format=raw,index=0,media=disk",
        disk_image_file.display()
    );
    cmd.arg("-drive").arg(extra_args);
    let out = cmd.output().unwrap();

    // Print Results
    let stdout = String::from_utf8(out.stdout).unwrap();
    let stderr = String::from_utf8(out.stderr).unwrap();
    println!("{}\n{}", stdout, stderr);
}

#[test]
fn test_images_correct_size() {
    assert_sizes();
}
