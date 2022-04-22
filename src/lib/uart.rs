use crate::{memory, Board};

#[derive(Clone, Copy)]
pub enum Port {
    Zero = 0,
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    Seven = 7,
}
pub struct PortOptions {
    pub baud_rate: u32, // TODO: right type?
}

impl Port {
    /// The starting point of memory addresses corresponding to this GPIO register
    ///
    /// Modeled after page 904 of data sheet (UART Register Map)
    const fn base(&self) -> u32 {
        match self {
            Port::Zero => 0x4000_C000,
            Port::One => 0x4000_D000,
            Port::Two => 0x4000_E000,
            Port::Three => 0x4000_F000,
            Port::Four => 0x4001_0000,
            Port::Five => 0x4001_1000,
            Port::Six => 0x4001_2000,
            Port::Seven => 0x4001_3000,
        }
    }

    /// The memory address of the control (CTL) register for this port
    ///
    /// Page 918 of data sheet
    pub(super) const fn control(&self) -> *mut u32 {
        const OFFSET: u32 = 0x030;
        (self.base() + OFFSET) as *mut u32
    }

    /// The memory address of the data (DR) register for this port
    ///
    /// Page 906 of data sheet
    pub(super) const fn data(&self) -> *mut u32 {
        const OFFSET: u32 = 0x000;
        (self.base() + OFFSET) as *mut u32
    }

    /// The memory address of the flag (FR) register for this port
    ///
    /// Page 911 of data sheet
    pub(super) const fn flag(&self) -> *mut u32 {
        const OFFSET: u32 = 0x018;
        (self.base() + OFFSET) as *mut u32
    }

    /// The memory address of the TODO
    ///
    /// Page TODO of data sheet
    pub(super) const fn fractional_baud_rate_divisor(&self) -> *mut u32 {
        const OFFSET: u32 = 0x018; // TODO
        (self.base() + OFFSET) as *mut u32
    }

    /// The memory address of the TODO
    ///
    /// Page TODO of data sheet
    pub(super) const fn integer_baud_rate_divisor(&self) -> *mut u32 {
        const OFFSET: u32 = 0x018; // TODO
        (self.base() + OFFSET) as *mut u32
    }
}

impl Port {
    /// The enable (EN) bit in the control register
    const fn enable_bit(&self) -> u32 {
        0
    }

    /// The system's Run mode clock gating control (RCGC) register address containing this port
    fn run_mode_clock_gate_control_address(&self, board: Board) -> *mut u32 {
        match self {
            Port::Zero => board.run_mode_clock_gate_control_1(),
            Port::One => board.run_mode_clock_gate_control_1(),
            Port::Two => board.run_mode_clock_gate_control_1(),
            _ => todo!(),
        }
    }
    /// The corresponding bit for this port in the system's Run mode clock gating control (RCGC) register
    fn run_mode_clock_gate_control_bit(&self) -> u32 {
        match self {
            Port::Zero => 0,
            Port::One => 1,
            Port::Two => 2,
            _ => todo!(),
        }
    }
}

pub struct UsablePort {
    port: Port,
}

impl UsablePort {
    pub fn do_something() {
        todo!();
    }
}

pub fn setup_port(board: Board, port: Port, options: PortOptions, no_op: &dyn Fn()) -> UsablePort {
    // Activate the associated peripheral
    unsafe {
        memory::set_bits(
            board.run_mode_clock_gate_control_1(),
            &[port.run_mode_clock_gate_control_bit()],
        );
    }

    // Page 904: There must be a delay of 3 system clocks after the UART module clock is enabled before any UART module registers are accessed.
    // But for some reason, 7 (not 3) no-ops are needed
    for _ in 0..7 {
        no_op();
    }

    // Disable this UART port while setting it up
    unsafe {
        memory::clear_bits(port.control(), &[port.enable_bit()]);
    }

    // Page 896: baud rate generation

    // page 219
    /// 16 MHz
    const SYSTEM_OSC_CLOCK_SPEED: u32 = 16_000_000;
    // the MOSC is variable frequeny (5 MHz to 25 MHz)

    // the XOSC can act as a real time clock as well!

    // The internal system clock (SysClk), is derived from any of the above sources plus two others: the
    // output of the main internal PLL and the precision internal oscillator divided by four (4 MHz ± 1%).
    // The frequency of the PLL clock reference must be in the range of 5 MHz to 25 MHz (inclusive).
    // Table 5-3 on page 220 shows how the various clock sources can be used in a system
    // TODO: migrate all of the above comments to a github issue

    // TODO: how do you determine what's being used as the system clock?!
    let system_clock = SYSTEM_OSC_CLOCK_SPEED;    


    // TODO: The UART generates an internal baud-rate reference clock at 8x or 16x the baud-rate (referred to
    // as Baud8 and Baud16, depending on the setting of the HSE bit (bit 5) in UARTCTL)
    let clock_divider = 16;

    let baud_rate_divisor = (system_clock as f32) / ((clock_divider * options.baud_rate) as f32);

    let baud_rate_divisor_integer = baud_rate_divisor as u32;
    let baud_rate_divisor_fraction = baud_rate_divisor - (baud_rate_divisor_integer as f32);

    // TODO:
    // if baud_rate_divisor_integer.to_bits() > 22 {
    //     panic!();
    // }

    let baud_rate_divisor_fraction = ((baud_rate_divisor_fraction * 64.0) + 0.5) as u8;

    
    // TODO: verify and comment
    unsafe {
        memory::write(port.integer_baud_rate_divisor(), baud_rate_divisor_integer);
        memory::write(port.fractional_baud_rate_divisor(), baud_rate_divisor_fraction as u32);
    }

    // TODO: CTL LCHR register
    todo!();

    // Enable this UART port
    unsafe {
        memory::set_bits(port.control(), &[port.enable_bit()]);
    }

    UsablePort { port }
}
