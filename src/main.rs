use std::path::Path;

const BOOT_0: &[u8] = include_bytes!(env!("BIOS_STAGE0"));
const BOOT_1: &[u8] = include_bytes!(env!("BIOS_STAGE1"));
const BOOT_2: &[u8] = include_bytes!(env!("BIOS_STAGE2"));
const EXTRA_BYTES: [u8; 512] = [0; 512];

fn main() {
    assert_eq!(BOOT_0.len(), 512, "boot entry point was not correct size");
    assert_eq!(
        BOOT_1.len(),
        512 * config::REAL_MODE_SECTIONS,
        "boot protected was not correct size"
    );
    assert_eq!(BOOT_2.len(), 512 * 5, "boot protected was not correct size");

    let disk_bytes: Vec<u8> = BOOT_0
        .iter()
        .chain(BOOT_1.iter())
        .chain(BOOT_2.iter())
        .chain(EXTRA_BYTES.iter())
        .cloned()
        .collect();

    let disk_image_file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("disk.img");

    std::fs::write(disk_image_file, &disk_bytes).unwrap();
}
