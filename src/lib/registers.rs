//! Memory addresses of registers
//! Data sheet: https://www.ti.com/lit/ds/spms376e/spms376e.pdf

// TODO: check page 92-94 for more features ("Memory Map" table)!

// TODO: check page 1230 onward for PWM

/// Page 231 of data sheet
pub mod system {
    const BASE: u32 = 0x400F_E000;

    // TODO: page 340
    pub const RCGCGPIO: *mut u32 = (BASE + 0x608) as *mut u32;
}
