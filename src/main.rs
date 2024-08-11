use std::path::Path;

const BOOT_BYTES: &[u8] = include_bytes!(env!("BIOS_ENTRY"));
const EXTRA_BYTES: [u8; 512] = [b'A'; 512];

fn main() {
    println!("got bios of length === {}", BOOT_BYTES.len());
    let disk_bytes: Vec<u8> = BOOT_BYTES
        .iter()
        .chain(EXTRA_BYTES.iter())
        .cloned()
        .collect();

    let disk_image_file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("disk.img");

    std::fs::write(disk_image_file, &disk_bytes).unwrap();
}
