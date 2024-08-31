use common::println;
use core::arch::asm;
impl VbeInfoBlock {
    fn check() -> Result<(), VbeError> {
        todo!()
    }
    pub fn display(&self) -> VbeDisplay {
        self.into()
    }
}

#[allow(dead_code)]
#[repr(C, packed)]
pub struct VbeInfoBlock {
    // b"VESA" or [86, 69, 83, 65] or [0x56, 0x45, 0x53, 0x41]
    signature: [u8; 4],
    // 0x300 for VBE 3
    version: u16,
    // Points to a string
    oem_string_ptr: u32,
    capabillities: [u8; 4],
    video_mode_ptr: u32,
    total_memory: u16,
    reserved: [u8; 492],
}

#[test]
fn test_vbe_info_block_size() {
    assert_eq!(size_of::<VbeInfoBlock>(), 512);
}

#[test]
fn test_vbe_mode_info_block_size() {
    assert_eq!(size_of::<VesaModeInfoBlock>(), 256);
}

#[allow(dead_code)]
#[repr(C, packed)]
pub struct VesaModeInfoBlock {
    mode_attributes: u16,
    window_a: u8,
    window_b: u8,
    granularity: u16,
    window_size: u16,
    segment_a: u16,
    segment_b: u16,
    win_func_ptr: u32,
    pitch: u16,
    width: u16,
    height: u16,
    w_char: u8,
    y_char: u8,
    planes: u8,
    bpp: u8,
    banks: u8,
    memory_model: u8,
    bank_size: u8,
    image_pages: u8,
    reserved0: u8,
    red_mask: u8,
    red_position: u8,
    green_mask: u8,
    green_position: u8,
    blue_mask: u8,
    blue_position: u8,
    reserved_mask: u8,
    reserved_position: u8,
    direct_color_attributes: u8,
    framebuffer: u32,
    off_screen_mem_off: u32,
    off_screen_mem_size: u16,
    reserved1: [u8; 206],
}

impl<'a> From<&'a VbeInfoBlock> for VbeDisplay<'a> {
    fn from(value: &'a VbeInfoBlock) -> Self {
        VbeDisplay {
            signature: &value.signature,
            version: value.version,
            oem_string_ptr: value.oem_string_ptr,
            capabillities: &value.capabillities,
            video_mode_ptr: value.video_mode_ptr,
            total_memory: value.total_memory,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct VbeDisplay<'a> {
    // b"VESA" or [86, 69, 83, 65] or [0x56, 0x45, 0x53, 0x41]
    signature: &'a [u8; 4],
    // 0x300 for VBE 3
    version: u16,
    // Points to a string
    oem_string_ptr: u32,
    capabillities: &'a [u8; 4],
    video_mode_ptr: u32,
    total_memory: u16,
}

impl core::fmt::Display for VbeInfoBlock {
    fn fmt(&self, _f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        todo!()
    }
}

enum VbeError {
    SignatureNotValid,
}

/// SAFETY: Write to address 0x7000
pub unsafe fn get_vbe_info() {
    let vbe_info_pointer: *mut VbeInfoBlock = 0x7000 as *mut VbeInfoBlock;

    // Load VBE Info block
    {
        let magic_return_code = 0x4f;
        unsafe {
            let di: u16 = vbe_info_pointer as u16;
            let mut ax: u16 = 0x4f00;
            asm!(
                "int 0x10",
                inout("ax") ax,
                in("di") di
            );

            let ah = ax >> 8;
            let al = ax & 0x00ff;
            if al != magic_return_code || ah != 0 {
                panic!("vbe fail code");
            }
        }
    }

    unsafe { println!("{:#x?}", vbe_info_pointer.read().display()) }
}
