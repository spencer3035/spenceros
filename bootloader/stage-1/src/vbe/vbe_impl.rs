use core::arch::asm;
use core::mem::MaybeUninit;

use super::FramebufferInfo;
pub type Font = [u8; 0x1000];

pub fn init() -> FramebufferInfo {
    assert_eq!(size_of::<VesaVbeBlockDef>(), 512, "VbeInfoBlock bad size");
    assert_eq!(
        size_of::<VesaVbeModeDef>(),
        256,
        "VesaModeInfoBlock bad size"
    );
    set_best_vbe_mode()
}

/// Loads BIOS VGA font into a given address
pub fn set_bitmap_font_from_bios(font: &mut Font) {
    // ES:BP is address of font we want to save
    let mut bp: u16;
    let mut es: u16;
    unsafe {
        asm!(
            // Save segment register, they get modified by bios call
            "push			es",
            // Ask BIOS to return VGA bitmap font location
            //
            // Returns pointer to font at ES:BP, as well as info in CX and DL we don't care about
            "mov			ax, 1130h",
            "mov			bh, 6",
            "int			0x10",
            // Save results
            "mov			{0:x}, bp",
            "mov			{1:x}, es",
            // Reset segment register
            "pop			es",
            out(reg) bp,
            out(reg) es,
        );
    }

    // Convert segmented addressing to linear address
    let address = (16 * (es as usize) + bp as usize) as *const u8;

    // Save font
    for ii in 0..0x1000 {
        unsafe {
            font[ii] = address.add(ii).read();
        }
    }
}

/// Checks that the ax value indicates return success for VBE function calls. Panics if not success
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

/// Gets the best vbe mode given desired width, height, depth, and a list of supported mode ids
fn get_best_mode(width: u16, height: u16, depth: u8, modes: &[u16]) -> FramebufferInfo {
    let mut diff = u16::MAX;
    let mut best_mode = None;

    // SAFETY: This gets init with the load() function at the beginning of each loop. If it
    // doesn't enter the loop, best_mode will be none and will panic before using any info from
    // this variable
    let mut vbe_mode: FramebufferInfo = unsafe { MaybeUninit::uninit().assume_init() };

    for mode_id in modes.iter() {
        if let Err(_) = vbe_mode.load(*mode_id) {
            continue;
        }
        // Check the residual
        let mode_diff = vbe_mode.width.abs_diff(width) + vbe_mode.height.abs_diff(height);
        if vbe_mode.bits_per_pixel == depth && mode_diff <= diff {
            diff = mode_diff;
            best_mode = Some(*mode_id);
        }
    }

    if modes.is_empty() || diff == u16::MAX || best_mode.is_none() {
        panic!("no VBE modes found");
    }
    let best_mode = best_mode.unwrap();

    if let Err(e) = vbe_mode.load(best_mode) {
        panic!("Couldn't load VBE mode: {e:?}");
    }

    // Read mode to structure
    vbe_mode.load(best_mode).unwrap();
    vbe_mode
}

/// SAFETY: Can only be called by one thread at a time, contains mutable static information
fn set_best_vbe_mode() -> FramebufferInfo {
    // Get the best mode relative to these target numbers
    // TODO: Get these numbers from EDID: https://wiki.osdev.org/EDID
    //let (width, height) = (1920, 1080, 24);
    let (width, height, depth) = (1280, 720, 24);

    let vbe_block = VesaVbeBlockDef::new();
    let modes = vbe_block.get_modes();
    let best_mode = get_best_mode(width, height, depth, modes);

    const USE_LINEAR_FRAME_BUFFER: u16 = 0x4000;
    #[allow(dead_code)]
    const USE_CRTC_INFO_BLOCK: u16 = 1 << 10;
    // Set the mode
    unsafe {
        let mut ax = 0x4f02;
        asm!(
            "int 0x10",
            inout("ax") ax,
            in("bx") best_mode.mode_id | USE_LINEAR_FRAME_BUFFER
            // in("bx") mode_id | USE_LINEAR_FRAME_BUFFER | USE_CRTC_INFO_BLOCK
            // in("di") &CRTCInfoBlock
        );
        // This doesn't display properly if the function succeeds, if it failes it presumibly would
        // display correctly because the mode would be the same.
        check_vbe_ax!(ax, "VBE load fail code : 0x{ax:x}");
    }

    best_mode
}

/// Defininition/memory layout for the Vesa VBE info block 3.0
#[allow(dead_code)]
#[repr(C, packed)]
pub struct VesaVbeBlockDef {
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

impl VesaVbeBlockDef {
    fn get_modes(&self) -> &[u16] {
        let mode_ptr = self.video_mode_ptr as *const u16;
        let max_modes = 0x100;
        let mut length = 0;
        while unsafe { mode_ptr.add(length).read() != 0xffff } && length < max_modes {
            length += 1;
        }

        if length == max_modes {
            panic!("Didn't hit end of modes list");
        }

        unsafe { core::slice::from_raw_parts(mode_ptr, length) }
    }

    /// Loads VBE into new structure
    fn new() -> Self {
        // SAFETY: result is init by the assembly call. It is additionally checked for validity
        // after and panics if invalid
        let mut res: Self = unsafe { MaybeUninit::uninit().assume_init() };
        // Modifies the content of self
        unsafe {
            let mut ax: u16 = 0x4f00;
            asm!(
                "int 0x10",
                inout("ax") ax,
                in("di") &mut res
            );
            check_vbe_ax!(ax, "VBE load fail code 0x{ax:x}");
        }

        res.check().unwrap();
        res
    }

    /// Checks if block is valid
    fn check(&self) -> Result<(), VbeError> {
        if &self.signature != b"VESA" {
            // Check signature
            Err(VbeError::SignatureNotValid)
        } else if self.version != 0x300 {
            // Check version
            Err(VbeError::NotVerson3)
        } else if self.capabillities != [1, 0, 0, 0] {
            common::println_bios!("self={:?}", self.capabillities);
            common::println_bios!("arr ={:?}", [1, 0, 0, 0]);
            for (ii, val) in self.capabillities.iter().enumerate() {
                common::println_bios!("{ii}: {val}");
            }
            // Check capabilities are as expected
            Err(VbeError::BadCapabilities)
        } else {
            Ok(())
        }
    }
}

impl core::fmt::Display for VesaVbeBlockDef {
    fn fmt(&self, _f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        todo!()
    }
}

impl FramebufferInfo {
    /// Reads a VBE mode to frame buffer
    fn load(&mut self, mode_id: u16) -> Result<(), VbeError> {
        // SAFETY: vbe is populated with bios call below and checked for validity immediately after
        let mut vbe_mode_def: VesaVbeModeDef = unsafe { MaybeUninit::uninit().assume_init() };
        let mut ax = 0x4f01;
        unsafe {
            asm!(
                "int 0x10",
                inout("ax") ax,
                in("cx") mode_id,
                in("di") &mut vbe_mode_def,
            );
        }
        check_vbe_ax!(ax, "VBE mode fail");
        vbe_mode_def.check()?;

        // Check it is a mode we want
        // Packed pixel or direct color
        let memory_model_works = vbe_mode_def.memory_model == 4 || vbe_mode_def.memory_model == 6;
        let required_flags =
            SUPPORTED_BY_HARDWARE | LINEAR_FRAME_BUFFER | NO_VGA_COMPAT | GRAPICS_MODE;
        let has_flags = vbe_mode_def.mode_attributes & required_flags == required_flags;
        let good_mode = memory_model_works && has_flags;
        if !good_mode {
            return Err(VbeError::ModeNotGood);
        }

        *self = FramebufferInfo {
            mode_id,
            bits_per_pixel: vbe_mode_def.bits_per_pixel,
            bytes_per_scan_line: vbe_mode_def.bytes_per_scan_line,
            width: vbe_mode_def.width,
            height: vbe_mode_def.height,
            framebuffer: vbe_mode_def.framebuffer as *mut u8,
        };

        Ok(())
    }
}

/// Defininition/memory layout for the VesaVbeMode 3.0
#[derive(Debug)]
#[allow(dead_code)]
#[repr(C, packed)]
struct VesaVbeModeDef {
    // ** Manditory for all VBE revisions
    mode_attributes: u16,
    window_a: u8,
    window_b: u8,
    granularity: u16,
    window_size: u16,
    segment_a: u16,
    segment_b: u16,
    win_func_ptr: u32,
    // ** Manditory for VBE 1.2 and above
    /// Number of bytes used per pixel
    bytes_per_scan_line: u16,
    /// How many pixels wide the screen is
    width: u16,
    /// How many pixels high the screen is
    height: u16,
    w_char: u8,
    y_char: u8,
    planes: u8,
    /// How many bits per pixes, should be 4 or 6
    bits_per_pixel: u8,
    banks: u8,
    memory_model: u8,
    bank_size: u8,
    image_pages: u8,
    reserved0: u8,
    // ** Direct color fields for memory models 6 and 7 **
    red_mask: u8,
    red_position: u8,
    green_mask: u8,
    green_position: u8,
    blue_mask: u8,
    blue_position: u8,
    reserved_mask: u8,
    reserved_position: u8,
    direct_color_attributes: u8,
    // ** Mandatory information for VBE 2.0 and above
    /// Start address of the framebuffer
    framebuffer: u32,
    off_screen_mem_off: u32,
    off_screen_mem_size: u16,
    // ** Mandatory information for VBE 3.0 and above
    linear_bytes_per_scan_line: u16,
    bank_images_pages: u8,
    linear_images_pages: u8,
    linear_red_mask_size: u8,
    linear_red_field_pos: u8,
    linear_green_mask_size: u8,
    linear_green_field_pos: u8,
    linear_blue_mask_size: u8,
    linear_blue_field_pos: u8,
    linear_rsv_mask_size: u8,
    linear_rsv_field_pos: u8,
    max_pixel_clock: u32,
    // TODO: Spec says this should be 189
    reserved1: [u8; 190],
}

/// Mode is suported by hardware
const SUPPORTED_BY_HARDWARE: u16 = 1 << 0;
#[allow(dead_code)]
/// TTY Output supoprted by BIOS
const TTY_BIOS_OUT: u16 = 1 << 2;
#[allow(dead_code)]
/// Color mode is enabled
const COLOR_MODE: u16 = 1 << 3;
/// Graphics mode, not text mode
const GRAPICS_MODE: u16 = 1 << 4;
/// Doesn't have VGA compatability
const NO_VGA_COMPAT: u16 = 1 << 5;
#[allow(dead_code)]
/// Has VGA compatible windowed memory
const WINDOWED_MEMORY: u16 = 1 << 6;
/// Has linear frame buffer
const LINEAR_FRAME_BUFFER: u16 = 1 << 7;

impl VesaVbeModeDef {
    fn check(&self) -> Result<(), VbeError> {
        // TODO: Check for validity
        Ok(())
    }
}

impl core::fmt::Display for VesaVbeModeDef {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // This is needed to make rust not complain about packed fields being unaligned
        let (w, h, bpp, attr) = (
            self.width,
            self.height,
            self.bits_per_pixel,
            self.mode_attributes,
        );
        write!(f, "{}x{}x{} attr = 0b{:b}", w, h, bpp, attr)?;
        Ok(())
    }
}

// TODO: Add something like this to display actual bad value
//struct VbeCompleteError<'a> {
//    vbe_block: &'a VbeInfoBlock,
//    error: VbeError,
//}

// TODO: Expand on errors
#[derive(Debug)]
pub enum VbeError {
    ModeNotGood,
    SignatureNotValid,
    NotVerson3,
    BadCapabilities,
}
