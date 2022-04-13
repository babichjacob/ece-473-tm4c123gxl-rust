//! Memory addresses of registers
//! Data sheet: https://www.ti.com/lit/ds/spms376e/spms376e.pdf

// TODO: check page 92-94 for more features ("Memory Map" table)!

// TODO: check page 1230 onward for PWM

/// Modeled after page 660 of data sheet (GPIO Register Map)
pub mod gpio {
    mod base {
        pub const PORT_A: u32 = 0x4000_4000;
        pub const PORT_F: u32 = 0x4002_5000;
    }

    /// Page 671 of data sheet
    pub mod afsel {
        const OFFSET: u32 = 0x420;

        pub const PORT_A: *mut u32 = (super::base::PORT_A + OFFSET) as *mut u32;
        pub const PORT_F: *mut u32 = (super::base::PORT_F + OFFSET) as *mut u32;
    }

    /// Page 687 of data sheet
    pub mod amsel {
        const OFFSET: u32 = 0x52C;

        pub const PORT_A: *mut u32 = (super::base::PORT_A + OFFSET) as *mut u32;
        pub const PORT_F: *mut u32 = (super::base::PORT_F + OFFSET) as *mut u32;
    }

    /// Page 685 of data sheet
    pub mod cr {
        const OFFSET: u32 = 0x524;

        pub const PORT_A: *mut u32 = (super::base::PORT_A + OFFSET) as *mut u32;
        pub const PORT_F: *mut u32 = (super::base::PORT_F + OFFSET) as *mut u32;
    }

    /// Page 662 of data sheet
    pub mod data {
        const OFFSET: u32 = 0x000;

        pub const PORT_A: *mut u32 = (super::base::PORT_A + OFFSET) as *mut u32;
        pub const PORT_F: *mut u32 = (super::base::PORT_F + OFFSET) as *mut u32;
    }

    /// Page 682 of data sheet
    pub mod den {
        const OFFSET: u32 = 0x51C;

        pub const PORT_A: *mut u32 = (super::base::PORT_A + OFFSET) as *mut u32;
        pub const PORT_F: *mut u32 = (super::base::PORT_F + OFFSET) as *mut u32;
    }

    /// Page 663 of data sheet
    pub mod dir {
        const OFFSET: u32 = 0x400;

        pub const PORT_A: *mut u32 = (super::base::PORT_A + OFFSET) as *mut u32;
        pub const PORT_F: *mut u32 = (super::base::PORT_F + OFFSET) as *mut u32;
    }

    /// Page 684 of data sheet
    pub mod lock {
        const OFFSET: u32 = 0x520;

        pub const PORT_A: *mut u32 = (super::base::PORT_A + OFFSET) as *mut u32;
        pub const PORT_F: *mut u32 = (super::base::PORT_F + OFFSET) as *mut u32;
    }

    /// Page 688 of data sheet
    pub mod pctl {
        const OFFSET: u32 = 0x52C;

        pub const PORT_A: *mut u32 = (super::base::PORT_A + OFFSET) as *mut u32;
        pub const PORT_F: *mut u32 = (super::base::PORT_F + OFFSET) as *mut u32;
    }

    /// Page 679 of data sheet
    pub mod pdr {
        const OFFSET: u32 = 0x514;

        pub const PORT_A: *mut u32 = (super::base::PORT_A + OFFSET) as *mut u32;
        pub const PORT_F: *mut u32 = (super::base::PORT_F + OFFSET) as *mut u32;
    }

    /// Page 677 of data sheet
    pub mod pur {
        const OFFSET: u32 = 0x510;

        pub const PORT_A: *mut u32 = (super::base::PORT_A + OFFSET) as *mut u32;
        pub const PORT_F: *mut u32 = (super::base::PORT_F + OFFSET) as *mut u32;
    }

    // TODO: examine page 670 for when (if) I do interrupts
}

// TODO: examine page 690 (ADC) for applicability

/// Page 231 of data sheet
pub mod system {
    const BASE: u32 = 0x400F_E000;

    // TODO: page 340
    pub const RCGCGPIO: *mut u32 = (BASE + 0x608) as *mut u32;
}
