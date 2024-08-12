use std::path::Path;

use config::{STAGE_1_SECTIONS, STAGE_2_SECTIONS};

const BOOT_0: &[u8] = include_bytes!(env!("BIOS_STAGE0"));
const BOOT_1: &[u8] = include_bytes!(env!("BIOS_STAGE1"));
const BOOT_2: &[u8] = include_bytes!(env!("BIOS_STAGE2"));
const EXTRA_BYTES: [u8; 512] = [0; 512];

fn main() {
    // If this fails, need to read more sectors in stage 0 or 1
    let total_sectors = STAGE_1_SECTIONS + STAGE_2_SECTIONS;
    assert!(
        total_sectors < u8::MAX as usize,
        "too many sectors to read with one u8"
    );

    // First section needs to always be 512 bytes
    assert_eq!(BOOT_0.len(), 512, "boot entry point was not correct size");
    // Check section 1 is the correct size
    assert_eq!(
        BOOT_1.len(),
        512 * config::STAGE_1_SECTIONS,
        "boot protected was not correct size"
    );
    // Check section 2 is the correct size
    assert_eq!(
        BOOT_2.len(),
        512 * config::STAGE_2_SECTIONS,
        "boot protected was not correct size"
    );

    // Put all sections together
    let disk_bytes: Vec<u8> = BOOT_0
        .iter()
        .chain(BOOT_1.iter())
        .chain(BOOT_2.iter())
        .chain(EXTRA_BYTES.iter())
        .cloned()
        .collect();

    // Write to file
    let disk_image_file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("disk.img");
    std::fs::write(disk_image_file, &disk_bytes).unwrap();
}
