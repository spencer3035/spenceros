use common::println;
use core::{arch::asm, ptr::addr_of_mut};

static mut VBE_BLOCK: VbeInfoBlock = VbeInfoBlock::null();
static mut VBE_MODE: VesaModeInfoBlock = VesaModeInfoBlock::null();

macro_rules! check_vbe_ax {
    ($ax:ident, $($args:tt)*) => {
            let ah = $ax >> 8;
            let al = $ax & 0x00ff;
            // 0x4f is magic return code
            if al != 0x4f || ah != 0 {
                panic!($($args)*)
            }

    };
}

/// Enters the best fit VBE mode
///
/// SAFETY: Writes to static variables, can't be used accross threads
pub unsafe fn enter_vbe_mode() -> (VbeInfoBlock, VesaModeInfoBlock) {
    assert_eq!(size_of::<VbeInfoBlock>(), 512, "VbeInfoBlock bad size");
    assert_eq!(
        size_of::<VesaModeInfoBlock>(),
        256,
        "VesaModeInfoBlock bad size"
    );

    unsafe { set_best_vbe_mode() }
}

/// Loads the static [`VbeInfoBlock`]
///
/// SAFETY: Sets and returns reference to static VbeInfoBlock, can't be used across threads
unsafe fn load_info_block() -> VbeInfoBlock {
    let vbe_block: *mut VbeInfoBlock = addr_of_mut!(VBE_BLOCK);
    // Load VBE Info block
    {
        // SAFETY: Mutates VBE_BLOCK, and VBE_BLOCK is marked as mutable
        unsafe {
            let mut ax: u16 = 0x4f00;
            asm!(
                "int 0x10",
                inout("ax") ax,
                in("di") vbe_block
            );

            check_vbe_ax!(ax, "VBE load fail code");
        }
    }

    unsafe {
        let blk = vbe_block.read();
        blk.check().unwrap();
        blk
    }
}

/// Gets the best vbe mode given width, and height.
///
/// Returns mode number and structure of the mode
///
/// SAFETY: Modifies static variables, can not be used across threads
unsafe fn get_best_mode(width: u16, height: u16, info: &VbeInfoBlock) -> (VesaModeInfoBlock, u16) {
    // TODO: Switch to maybe uninit and declare init after int 0x10 call
    let vbe_mode = addr_of_mut!(VBE_MODE);

    let modes: *const u16 = info.video_mode_ptr as *const u16;
    let mut diff = u16::MAX;
    let mut best_mode = 0;

    let mut num_modes = 0;
    // This could probably be smaller
    let max_modes = 0x100;
    loop {
        let mode_id: u16 = unsafe { modes.add(num_modes).read() };
        // Last mode
        if mode_id == 0xffff {
            break;
        }
        num_modes += 1;

        // Read mode to structure
        let mut ax = 0x4f01;
        unsafe {
            asm!(
                "int 0x10",
                inout("ax") ax,
                in("cx") mode_id,
                in("di") vbe_mode,
            );
        }
        check_vbe_ax!(ax, "VBE mode fail");

        let vbe_mode: VesaModeInfoBlock = unsafe {
            let vbe_mode = vbe_mode.read();
            vbe_mode.check().unwrap();
            vbe_mode
        };

        // Check the residual
        let mode_diff = vbe_mode.width.abs_diff(width) + vbe_mode.height.abs_diff(height);
        let good_mode = vbe_mode.memory_model == 4 || vbe_mode.memory_model == 6;

        if !good_mode {
            continue;
        }

        if mode_diff < diff {
            diff = mode_diff;
            best_mode = mode_id;
        }

        if diff == 0 {
            break;
        }

        if num_modes > max_modes {
            panic!("last mode not found");
        }
    }

    if num_modes == 0 || diff == u16::MAX {
        panic!("no VBE modes found");
    }

    unsafe {
        println!("Best mode = {}", vbe_mode.read());
    }

    unsafe {
        let blk = vbe_mode.read();
        blk.check().unwrap();
        (blk, best_mode)
    }
}

/// SAFETY: Can only be called by one thread at a time, contains mutable static information
unsafe fn set_best_vbe_mode() -> (VbeInfoBlock, VesaModeInfoBlock) {
    // Get the best mode relative to these target numbers
    // TODO: Get these numbers from EDID: https://wiki.osdev.org/EDID
    let width = 720;
    let height = 480;

    unsafe {
        let info_block = load_info_block();
        let (mode_block, mode_id) = get_best_mode(width, height, &info_block);

        const USE_LINEAR_FRAME_BUFFER: u16 = 0x4000;
        const USE_CRTC_INFO_BLOCK: u16 = 1 << 10;
        // Set the mode
        let mut ax = 0x4f02;
        asm!(
        "int 0x10",
        inout("ax") ax,
        in("bx") mode_id | USE_LINEAR_FRAME_BUFFER
        // in("bx") mode_id | USE_LINEAR_FRAME_BUFFER | USE_CRTC_INFO_BLOCK
        // in("di") &CRTCInfoBlock
        );
        check_vbe_ax!(ax, "VBE load fail code");

        (info_block, mode_block)
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
    oem_software_rev: u16,
    oem_vendor_name_ptr: u32,
    oem_product_name_ptr: u32,
    oem_product_rev_ptr: u32,
    reserved: [u8; 222],
    oem_data: [u8; 256],
}

impl VbeInfoBlock {
    const fn null() -> VbeInfoBlock {
        VbeInfoBlock {
            signature: [0; 4],
            version: 0,
            oem_string_ptr: 0,
            capabillities: [0; 4],
            video_mode_ptr: 0,
            total_memory: 0,
            oem_software_rev: 0,
            oem_vendor_name_ptr: 0,
            oem_product_name_ptr: 0,
            oem_product_rev_ptr: 0,
            reserved: [0; 222],
            oem_data: [0; 256],
        }
    }
    fn check(&self) -> Result<(), VbeError> {
        if &self.signature != b"VESA" {
            // Check signature
            Err(VbeError::SignatureNotValid)
        } else if self.version != 0x300 {
            // Check version
            Err(VbeError::NotVerson3)
        } else if self.capabillities != [1, 0, 0, 0] {
            // Check capabilities are as expected
            Err(VbeError::BadCapabilities)
        } else {
            Ok(())
        }
    }
    fn display(&self) -> VbeDisplay {
        self.into()
    }
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

#[allow(dead_code)]
#[repr(C, align(16))]
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
    bits_per_pixel: u8,
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

impl VesaModeInfoBlock {
    fn check(&self) -> Result<(), VbeError> {
        // TODO: Check for validity
        Ok(())
    }
    const fn null() -> VesaModeInfoBlock {
        VesaModeInfoBlock {
            mode_attributes: 0,
            window_a: 0,
            window_b: 0,
            granularity: 0,
            window_size: 0,
            segment_a: 0,
            segment_b: 0,
            win_func_ptr: 0,
            pitch: 0,
            width: 0,
            height: 0,
            w_char: 0,
            y_char: 0,
            planes: 0,
            bits_per_pixel: 0,
            banks: 0,
            memory_model: 0,
            bank_size: 0,
            image_pages: 0,
            reserved0: 0,
            red_mask: 0,
            red_position: 0,
            green_mask: 0,
            green_position: 0,
            blue_mask: 0,
            blue_position: 0,
            reserved_mask: 0,
            reserved_position: 0,
            direct_color_attributes: 0,
            framebuffer: 0,
            off_screen_mem_off: 0,
            off_screen_mem_size: 0,
            reserved1: [0; 206],
        }
    }
}

impl core::fmt::Display for VesaModeInfoBlock {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        //writeln!(f, "window_size : {}", self.window_size);
        //writeln!(f, "pitch : {}", self.pitch);
        write!(f, "{}x{}x{}", self.width, self.height, self.bits_per_pixel)?;
        //writeln!(f, "bpp : {}", self.bpp);
        //writeln!(f, "off_screen_mem_off : {}", self.off_screen_mem_off);
        //writeln!(f, "off_screen_mem_size : {}", self.off_screen_mem_size);
        Ok(())
    }
}

// TODO: Add something like this to display actual bad value
//struct VbeCompleteError<'a> {
//    vbe_block: &'a VbeInfoBlock,
//    error: VbeError,
//}

#[derive(Debug)]
enum VbeError {
    SignatureNotValid,
    NotVerson3,
    BadCapabilities,
}

#[test]
fn test_vbe_info_block_size() {}

#[test]
fn test_vbe_mode_info_block_size() {}
