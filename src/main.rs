use std::path::Path;

const BOOT_BYTES: &[u8] = include_bytes!(env!("BIOS_ENTRY"));
const EXTRA_BYTES: [u8; 512 * 2] = [b'A'; 512 * 2];

fn main() {
    let mut disk_bytes: Vec<u8> = BOOT_BYTES
        .iter()
        .chain(EXTRA_BYTES.iter())
        .cloned()
        .collect();
    disk_bytes.push(b'z');

    let disk_image_file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("disk.img");

    std::fs::write(disk_image_file, &disk_bytes).unwrap();
}
