use std::path::Path;

const BOOT_ENTRY: &[u8] = include_bytes!(env!("BIOS_ENTRY"));
const BOOT_PROTECTED: &[u8] = include_bytes!(env!("BIOS_PROTECTED"));
const EXTRA_BYTES: [u8; 512] = [0; 512];

fn main() {
    assert_eq!(
        BOOT_ENTRY.len(),
        512,
        "boot entry point was not correct size"
    );
    assert_eq!(
        BOOT_PROTECTED.len(),
        512 * config::REAL_MODE_SECTIONS,
        "boot protected was not correct size"
    );

    let disk_bytes: Vec<u8> = BOOT_ENTRY
        .iter()
        .chain(BOOT_PROTECTED.iter())
        .chain(EXTRA_BYTES.iter())
        .cloned()
        .collect();

    let disk_image_file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("disk.img");

    std::fs::write(disk_image_file, &disk_bytes).unwrap();
}
