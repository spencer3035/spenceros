use super::Screen;
use common::println_bios as println;
use core::mem::MaybeUninit;
use core::{arch::asm, ptr::addr_of};

pub fn init() -> Screen {
    assert_eq!(size_of::<VesaVbeBlockDef>(), 512, "VbeInfoBlock bad size");
    assert_eq!(
        size_of::<VesaVbeModeDef>(),
        256,
        "VesaModeInfoBlock bad size"
    );
    unsafe {
        get_vga_font();
    }
    let mode = set_best_vbe_mode();

    Screen {
        width: mode.width,
        height: mode.height,
        depth: mode.bits_per_pixel,
        line_bytes: mode.bytes_per_scan_line,
        framebuffer: mode.framebuffer as *mut u8,
        font: unsafe { addr_of!(FONT).as_ref().unwrap() },
    }
}

static mut FONT: [u8; 0x1000] = [0; 0x1000];
/// Loads BIOS VGA font into static variable
///
/// SAFETY: Modifies static variable. Not thread safe
unsafe fn get_vga_font() {
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
            FONT[ii] = address.add(ii).read();
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
fn get_best_mode(width: u16, height: u16, depth: u8, modes: &[u16]) -> VesaModeInfo {
    let mut diff = u16::MAX;
    let mut best_mode = 0;

    let mut vbe_mode = VesaModeInfo::null();

    for mode_id in modes.iter() {
        if let Err(_) = vbe_mode.load(*mode_id) {
            continue;
        }
        // Check the residual
        let mode_diff = vbe_mode.width.abs_diff(width) + vbe_mode.height.abs_diff(height);
        if vbe_mode.bits_per_pixel == depth && mode_diff <= diff {
            diff = mode_diff;
            best_mode = *mode_id;
        }
    }

    if modes.is_empty() || diff == u16::MAX {
        panic!("no VBE modes found");
    }

    if let Err(e) = vbe_mode.load(best_mode) {
        panic!("Couldn't load VBE mode: {e:?}");
    }

    // Read mode to structure
    vbe_mode.load(best_mode).unwrap();
    vbe_mode
}

/// SAFETY: Can only be called by one thread at a time, contains mutable static information
fn set_best_vbe_mode() -> VesaModeInfo {
    // Get the best mode relative to these target numbers
    // TODO: Get these numbers from EDID: https://wiki.osdev.org/EDID
    //let (width, height) = (1920, 1080, 24);
    let (width, height, depth) = (1280, 720, 24);

    let vbe_block = VesaVbeBlockDef::new();
    let modes = vbe_block.get_modes();

    println!("modes = {modes:?}");

    // RefCell immutable
    let mode = get_best_mode(width, height, depth, modes);

    println!("Best mode {mode:?}");

    const USE_LINEAR_FRAME_BUFFER: u16 = 0x4000;
    #[allow(dead_code)]
    const USE_CRTC_INFO_BLOCK: u16 = 1 << 10;
    // Set the mode
    unsafe {
        let mut ax = 0x4f02;
        asm!(
            "int 0x10",
            inout("ax") ax,
            in("bx") mode.mode_id | USE_LINEAR_FRAME_BUFFER
            // in("bx") mode_id | USE_LINEAR_FRAME_BUFFER | USE_CRTC_INFO_BLOCK
            // in("di") &CRTCInfoBlock
        );
        // This doesn't display properly if the function succeeds, if it failes it presumibly would
        // display correctly because the mode would be the same.
        check_vbe_ax!(ax, "VBE load fail code : 0x{ax:x}");
    }

    mode
}

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

    /// Loads VBE info from block
    fn load(&mut self) {
        // Modifies the content of self
        unsafe {
            let mut ax: u16 = 0x4f00;
            asm!(
                "int 0x10",
                inout("ax") ax,
                in("di") self
            );
            check_vbe_ax!(ax, "VBE load fail code 0x{ax:x}");
        }

        self.check().unwrap();
    }

    /// Loads VBE into new structure
    fn new() -> Self {
        let mut res: MaybeUninit<Self> = MaybeUninit::uninit();
        unsafe {
            res.assume_init_mut().load();
            res.assume_init()
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
    #[allow(dead_code)]
    fn display(&self) -> VbeDisplay {
        self.into()
    }
}

impl<'a> From<&'a VesaVbeBlockDef> for VbeDisplay<'a> {
    fn from(value: &'a VesaVbeBlockDef) -> Self {
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

impl core::fmt::Display for VesaVbeBlockDef {
    fn fmt(&self, _f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        todo!()
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct VesaModeInfo {
    mode_id: u16,
    /// Number of bytes to get next horizontal row
    bytes_per_scan_line: u16,
    /// How many pixels wide the screen is
    width: u16,
    /// How many pixels high the screen is
    height: u16,
    /// How many bits per pixes, should be 4 or 6
    bits_per_pixel: u8,
    /// Start address of the framebuffer
    framebuffer: *mut u8,
    // TODO: Put color mask
}

impl VesaModeInfo {
    const fn null() -> VesaModeInfo {
        VesaModeInfo {
            mode_id: 0,
            bytes_per_scan_line: 0,
            width: 0,
            height: 0,
            bits_per_pixel: 0,
            framebuffer: 0 as *mut u8,
        }
    }
    fn load(&mut self, mode_id: u16) -> Result<(), VbeError> {
        // Read mode to structure
        //
        // SAFETY: Assembly modifies vbe_mode_def, needs to remain mutable to maintain correctness
        let mut vbe_mode_def = VesaVbeModeDef::null();
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

        *self = VesaModeInfo {
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

#[derive(Debug)]
#[allow(dead_code)]
#[repr(C, packed)]
pub struct VesaVbeModeDef {
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
    const fn null() -> VesaVbeModeDef {
        VesaVbeModeDef {
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

// TODO: Add something like this to display actual bad value
//struct VbeCompleteError<'a> {
//    vbe_block: &'a VbeInfoBlock,
//    error: VbeError,
//}

#[derive(Debug)]
pub enum VbeError {
    ModeNotGood,
    SignatureNotValid,
    NotVerson3,
    BadCapabilities,
}
