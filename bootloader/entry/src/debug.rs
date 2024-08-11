use core::arch::asm;
pub fn print_char(c: &u8) {
    let ax = *c as u16 | 0x0e00;
    unsafe {
        asm!(
            "int 0x10",
            in("ax") ax,
        );
    }
}

pub fn print_dec(mut num: u16) {
    let mut num_digits = 0;

    loop {
        let digit: u16 = num % 10;
        unsafe {
            asm!("push {0:x}", in(reg) digit);
        }
        num_digits += 1;
        num /= 10;
        if num == 0 {
            break;
        }
    }

    while num_digits >= 1 {
        let digit: i16;
        unsafe {
            asm!("pop {0:x}", out(reg) digit);
        }
        let value = digit as u8 + b'0';
        print_char(&value);
        num_digits -= 1;
    }
}

pub fn println(chars: &[u8]) {
    print(chars);
    print(b"\r\n");
}

pub fn print(chars: &[u8]) {
    for val in chars.iter() {
        print_char(val);
    }
}
