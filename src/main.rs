use std::path::Path;

use common::config::*;

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
        "boot stage 1 (0x{:x}) was not correct size (0x{:x})",
        BOOT_1.len() / 0x200,
        STAGE_1_SECTIONS,
    );
    // Check section 2 is the correct size
    assert_eq!(
        BOOT_2.len(),
        512 * STAGE_2_SECTIONS,
        "boot stage 2 (0x{:x}) was not correct size (0x{:x})",
        BOOT_2.len() / 0x200,
        STAGE_2_SECTIONS,
    );
    // Check section 3 is the correct size
    assert_eq!(
        BOOT_3.len(),
        512 * STAGE_3_SECTIONS,
        "boot stage 3 (0x{:x}) was not correct size (0x{:x})",
        BOOT_3.len() / 0x200,
        STAGE_3_SECTIONS,
    );

    // If this fails, need to read more sectors in stage 0 or 1
    let total_sectors = STAGE_0_SECTIONS + STAGE_1_SECTIONS + STAGE_2_SECTIONS + STAGE_3_SECTIONS;
    assert_eq!(
        total_sectors,
        SECTORS_TO_READ + 1,
        "Total sectors did not match expected"
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

#[cfg(test)]
mod test {
    use std::process::Stdio;

    use super::*;
    #[test]
    fn test_images_correct_size() {
        assert_sizes();
    }

    #[test]
    fn test_link_addresses() {
        let root_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let linker_files = [
            (
                "bootloader/stage-0/link.ld",
                STAGE_0_START as usize,
                size_of_val(unsafe { &(*STAGE_0_START) }),
            ),
            (
                "bootloader/stage-1/link.ld",
                STAGE_1_START as usize,
                size_of_val(unsafe { &(*STAGE_1_START) }),
            ),
            (
                "bootloader/stage-2/link.ld",
                STAGE_2_START as usize,
                size_of_val(unsafe { &(*STAGE_2_START) }),
            ),
            (
                "bootloader/stage-3/link.ld",
                STAGE_3_START as usize,
                size_of_val(unsafe { &(*STAGE_3_START) }),
            ),
        ];
        let paths: Vec<_> = linker_files
            .iter()
            .map(|(f, s, e)| {
                let mut path = root_dir.to_path_buf();
                path.push(f);
                (path, s, e)
            })
            .collect();
        for (p, start, size) in paths.into_iter() {
            test_link_file(p, *start, *size);
        }
    }

    fn test_link_file<P: AsRef<Path>>(p: P, start: usize, size: usize) {
        assert!(p.as_ref().try_exists().unwrap());
        let cmd = std::process::Command::new("ld")
            .arg("-T")
            .arg(p.as_ref())
            .arg("-M")
            .arg("/dev/null")
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();

        let out = String::from_utf8(cmd.wait_with_output().unwrap().stdout).unwrap();

        let get_addr_from_line = |line: &str| {
            let addr = line
                .split_whitespace()
                .next()
                .unwrap()
                .trim_start_matches("0x");
            let addr: usize = usize::from_str_radix(addr, 16).unwrap();
            addr
        };

        println!("Reading {}", p.as_ref().display());
        for line in out.lines() {
            if line.contains("_end_address") {
                let addr = get_addr_from_line(line);
                assert_eq!(
                    addr,
                    start + size,
                    "{} should have _end_address 0x{:X}, found 0x{addr:X}",
                    p.as_ref().display(),
                    start + size,
                );
            } else if line.contains("_start_address") {
                let addr = get_addr_from_line(line);
                assert_eq!(
                    addr,
                    start,
                    "{} should have _start_address 0x{:X}, found 0x{addr:X}",
                    p.as_ref().display(),
                    start,
                );
            }
        }
    }
}
