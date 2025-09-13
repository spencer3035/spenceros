use core::arch::asm;
use core::mem::MaybeUninit;
use core::ptr::addr_of;
use core::ptr::addr_of_mut;

use common::config::FONT;
use common::io::framebuffer::Color;
use common::io::framebuffer::FrameBuffer;
use common::io::framebuffer::FramebufferInfo;
use common::io::framebuffer::VbeDisplay;
use common::println_bios;
use common::println_vbe;
use common::BiosInfo;
use common::VbeDisplayInfo;

use crate::utils::get_stack_left;
use crate::utils::get_stack_used;
use crate::utils::prompt_continue;

/// Inits the VBE screen, should only be called once
pub fn init_graphical() {
    init();
}

fn init() {
    assert_eq!(size_of::<VesaVbeBlockDef>(), 512, "VbeInfoBlock bad size");
    assert_eq!(
        size_of::<VesaVbeModeDef>(),
        256,
        "VesaModeInfoBlock bad size"
    );
    init_font();
    init_framebuffer();
    VbeDisplay::init();
}

// TODO: Figure out why this causes things to print properly
fn fill_screen() {
    for ii in 0..VbeDisplay.width() {
        for jj in 0..VbeDisplay.height() {
            VbeDisplay.set_pixel(ii, jj, &Color::BLACK);
        }
    }
}

fn init_framebuffer() {
    let info = unsafe { VbeDisplayInfo::get_mut() };
    // Get the best mode relative to these target numbers
    let (width, height, depth) = get_preferred_width_height_depth();

    // SAFETY: This is the only time this function is called
    let vbe_block = unsafe { VesaVbeBlockDef::init_and_get() };
    let modes = vbe_block.get_modes();
    let best_mode = get_best_mode(width, height, depth, modes).unwrap();

    // SAFETY: framebuffer only exists within the scope
    unsafe {
        // Read best mode to structure
        let framebuffer = match load_framebuffer(best_mode) {
            Ok(f) => f,
            Err(e) => panic!("couldn't load mode {best_mode}: {e}"),
        };
        println_bios!("About to init graphical and clear screen");
        prompt_continue();
        set_vbe_mode(framebuffer);
        info.framebuffer = framebuffer.clone();
    }
}

/// Loads BIOS VGA font into a given address
fn init_font() {
    // ES:BP is address of font we want to save
    let mut bp: u16;
    let mut es: u16;
    unsafe {
        asm!(
            // Save segment register, they get modified by bios call
            "push es",
            // Ask BIOS to return VGA bitmap font location
            //
            // Returns pointer to font at ES:BP, as well as info in CX and DL we don't care about
            "mov ax, 1130h",
            "mov bh, 6",
            "int 0x10",
            // Save results
            "mov {0:x}, bp",
            "mov {1:x}, es",
            // Reset segment register
            "pop es",
            out(reg) bp,
            out(reg) es,
        );
    }

    // Convert segmented addressing to linear address
    let address = (16 * (es as usize) + bp as usize) as *const u8;
    let target: &mut [u8; 0x1000] = unsafe { (FONT as *mut [u8; 0x1000]).as_mut().unwrap() };

    // Save font
    for (ii, tgt) in target.iter_mut().enumerate() {
        unsafe {
            *tgt = address.add(ii).read();
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

static mut FRAME_BUFFER_INFO: FramebufferInfo = FramebufferInfo::null();

/// Gets the best vbe mode given desired width, height, depth, and a list of supported mode ids
fn get_best_mode(width: u16, height: u16, depth: u8, modes: &[u16]) -> Option<u16> {
    let mut diff = u16::MAX;
    let mut best_mode = None;

    for mode_id in modes.iter() {
        // SAFETY: framebuffer only exists within the scope
        unsafe {
            let framebuffer = match load_framebuffer(*mode_id) {
                Err(_) => {
                    continue;
                }
                Ok(f) => f,
            };
            // Check the residual
            let mode_diff = framebuffer.width.abs_diff(width) + framebuffer.height.abs_diff(height);
            if framebuffer.bits_per_pixel == depth && mode_diff <= diff {
                diff = mode_diff;
                best_mode = Some(*mode_id);
            }
        }
    }

    if modes.is_empty() || diff == u16::MAX || best_mode.is_none() {
        panic!("no VBE modes found");
    }
    best_mode
}

/// SAFETY: Can only be called by one thread at a time, contains mutable static information
fn set_vbe_mode(best_mode: &FramebufferInfo) {
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
}

#[derive(Debug)]
struct PreferredResolution {
    depth: u8,
    width: u16,
    height: u16,
}

/// Section 3.1 of doc
#[repr(C, align(0x80))]
#[derive(Debug)]
struct EdidData {
    // Header information
    header: [u8; 8],
    manufacturer_id: u16,
    product_id: u16,
    serial_id: u32,
    week: u8,
    year: u8,
    version: u8,
    revision: u8,
    // Basic display paramaters
    video_input_def: u8,
    horizontal_aspect_ratio: u8,
    vertical_aspect_ratio: u8,
    gamma: u8,
    feature_support: u8,
    // chromo corrds
    chromo_coords: [u8; 34 - 25 + 1],
    established_timing: [u8; 37 - 35 + 1],
    standard_timing: [u8; 53 - 38 + 1],
    display_timing: [u8; 125 - 54 + 1],
    extension_flag: [u8; 127 - 126 + 1],
}

impl EdidData {
    #[must_use]
    fn is_valid(&self) -> bool {
        // TODO: Convert to error instead of bool
        // TODO: Add more checks
        if self.header != [0, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0] {
            false
        } else {
            true
        }
    }

    const fn null() -> Self {
        Self {
            header: [0; 8],
            manufacturer_id: 0,
            product_id: 0,
            serial_id: 0,
            week: 0,
            year: 0,
            version: 0,
            revision: 0,
            video_input_def: 0,
            horizontal_aspect_ratio: 0,
            vertical_aspect_ratio: 0,
            gamma: 0,
            feature_support: 0,
            chromo_coords: [0; 34 - 25 + 1],
            established_timing: [0; 37 - 35 + 1],
            standard_timing: [0; 53 - 38 + 1],
            display_timing: [0; 125 - 54 + 1],
            extension_flag: [0; 127 - 126 + 1],
        }
    }
}

#[derive(Debug)]
struct EdidDataDisplay {
    header: [u8; 8],
    manufacturer_id: u16,
    product_id: u16,
    serial_id: u32,
    week: u8,
    year: u8,
    version: u8,
    revision: u8,
    video_input_def: u8,
    horizontal_aspect_ratio: u8,
    vertical_aspect_ratio: u8,
    gamma: u8,
    feature_support: u8,
}

static mut EDID_DATA: EdidData = EdidData::null();
fn get_preferred_width_height_depth() -> (u16, u16, u8) {
    assert_eq!(size_of::<EdidData>(), 0x80);

    let mut ax = 0x4f15;
    unsafe {
        // SAFETY: Edid data is init by bios call
        asm!(
            "mov bl, 0x01",
            "xor cx, cx",
            "xor dx, dx",
            "mov es, cx",
            "int 0x10",
            inout("ax") ax,
            in("di") addr_of_mut!(EDID_DATA),
        );
    };

    if ax != 0x4f {
        panic!("Bad ax : 0x{ax:x}");
    }

    unsafe {
        if !EDID_DATA.is_valid() {
            panic!("Bad edid data");
        }
    }

    let (def, info) = unsafe { (EDID_DATA.video_input_def, &EDID_DATA.display_timing) };
    let depth = if def & 0b10000000 != 0 {
        let bits_per_color = match (def & 0b01110000) >> 4 {
            0b001 => 6,
            0b010 => 8,
            0b011 => 10,
            0b100 => 12,
            0b101 => 14,
            0b110 => 16,
            _ => panic!("Unknown bit depth"),
        };
        // TODO: Check 3 colors per pixel (RGB).
        bits_per_color * 3
    } else {
        panic!("Analogue displays not supported");
    };
    let width = info[2] as u16 | ((info[4] & 0xF0) as u16) << 4;
    let height = info[5] as u16 | ((info[7] & 0xF0) as u16) << 4;

    (width, height, depth)
}

/// Static location to store the information at runtime
static mut VESA_VBE_BLOCK_DEF: VesaVbeBlockDef = VesaVbeBlockDef::null();

/// Defininition/memory layout for the Vesa VBE info block 3.0
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

    const fn null() -> Self {
        Self {
            // b"VESA" or [86, 69, 83, 65] or [0x56, 0x45, 0x53, 0x41]
            signature: [0; 4],
            // 0x300 for VBE 3
            version: 0,
            // Points to a string
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

    /// Loads VBE from BIOS and returns a reference to it
    ///
    /// # SAFETY: This should only be called once. It mutates a static variable and returns a
    unsafe fn init_and_get() -> &'static Self {
        let mut ax: u16 = 0x4f00;
        unsafe {
            // SAFETY: The layout of Self needs to match the spec
            // https://wiki.osdev.org/VESA_Video_Modes
            asm!(
                "int 0x10",
                inout("ax") ax,
                in("di") addr_of_mut!(VESA_VBE_BLOCK_DEF)
            );
        };

        check_vbe_ax!(ax, "VBE load fail code 0x{ax:x}");
        unsafe {
            VESA_VBE_BLOCK_DEF.check().unwrap();
            // SAFETY: Pointer should be aligned because it is declared as a static
            addr_of!(VESA_VBE_BLOCK_DEF).as_ref().unwrap()
        }
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
            // Check capabilities are as expected
            Err(VbeError::BadCapabilities)
        } else {
            Ok(())
        }
    }
}

static mut VBE_MODE_DEF: VesaVbeModeDef = VesaVbeModeDef::null();

/// Reads a VBE mode to frame buffer
///
/// # SAFETY: This uses a static variable to return references, so there should only be one
/// refernce to the return value at a time (don't call this function twice in the same scope or
/// deeper).
unsafe fn load_framebuffer(mode_id: u16) -> Result<&'static FramebufferInfo, VbeError> {
    let mut ax = 0x4f01;

    unsafe {
        // SAFETY: vbe is populated with bios call below and checked for validity immediately after
        asm!(
            "int 0x10",
            inout("ax") ax,
            in("cx") mode_id,
            in("di") addr_of_mut!(VBE_MODE_DEF)
        );
        VBE_MODE_DEF.check()?;
    }

    check_vbe_ax!(ax, "VBE mode fail");

    // Check it is a mode we want
    // Packed pixel or direct color
    let memory_model_works =
        unsafe { VBE_MODE_DEF.memory_model == 4 || VBE_MODE_DEF.memory_model == 6 };
    let required_flags = SUPPORTED_BY_HARDWARE | LINEAR_FRAME_BUFFER | NO_VGA_COMPAT | GRAPICS_MODE;
    let has_flags = unsafe { VBE_MODE_DEF.mode_attributes & required_flags == required_flags };
    let good_mode = memory_model_works && has_flags;
    if !good_mode {
        return Err(VbeError::ModeNotGood);
    }

    unsafe {
        FRAME_BUFFER_INFO = FramebufferInfo {
            mode_id,
            bits_per_pixel: VBE_MODE_DEF.bits_per_pixel,
            bytes_per_scan_line: VBE_MODE_DEF.bytes_per_scan_line,
            width: VBE_MODE_DEF.width,
            height: VBE_MODE_DEF.height,
            framebuffer: VBE_MODE_DEF.framebuffer as *mut u8,
        };
    }

    unsafe { Ok(addr_of!(FRAME_BUFFER_INFO).as_ref().unwrap()) }
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
        if self.framebuffer == 0 {
            return Err(VbeError::NullPointer);
        }
        Ok(())
    }

    const fn null() -> Self {
        Self {
            mode_attributes: 0,
            window_a: 0,
            window_b: 0,
            granularity: 0,
            window_size: 0,
            segment_a: 0,
            segment_b: 0,
            win_func_ptr: 0,
            bytes_per_scan_line: 0,
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
            linear_bytes_per_scan_line: 0,
            bank_images_pages: 0,
            linear_images_pages: 0,
            linear_red_mask_size: 0,
            linear_red_field_pos: 0,
            linear_green_mask_size: 0,
            linear_green_field_pos: 0,
            linear_blue_mask_size: 0,
            linear_blue_field_pos: 0,
            linear_rsv_mask_size: 0,
            linear_rsv_field_pos: 0,
            max_pixel_clock: 0,
            reserved1: [0; 190],
        }
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

// TODO: Expand on errors
#[derive(Debug)]
pub enum VbeError {
    ModeNotGood,
    SignatureNotValid,
    NotVerson3,
    BadCapabilities,
    NullPointer,
}

impl core::fmt::Display for VbeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            VbeError::ModeNotGood => write!(f, "invalid mode"),
            VbeError::SignatureNotValid => write!(f, "signature not VESA"),
            VbeError::NotVerson3 => write!(f, "not version 3"),
            VbeError::BadCapabilities => write!(f, "capabilities not supported"),
            VbeError::NullPointer => write!(f, "null pointer (not init?)"),
        }
    }
}
