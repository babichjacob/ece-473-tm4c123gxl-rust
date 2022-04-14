use crate::{
    memory, registers, Pin, ReadablePinOptions, ReadablePins, WritablePinOptions, WritablePins,
};

use super::pins::{setup_readable_pins, setup_writable_pins};

#[derive(Clone, Copy)]
pub enum Port {
    A,
    B,
    C,
    D,
    E,
    F,
}

pub struct PortOptions;

impl Port {
    /// The starting point of memory addresses corresponding to this GPIO register
    ///
    /// Modeled after page 660 of data sheet (GPIO Register Map)
    fn base(&self) -> u32 {
        match self {
            Port::A => 0x4000_4000,
            Port::B => 0x4000_5000,
            Port::C => 0x4000_6000,
            Port::D => 0x4000_7000,
            Port::E => 0x4002_4000,
            Port::F => 0x4002_5000,
        }
    }

    /// The memory address of the alternate function select (AFSEL) register for this port
    ///
    /// Page 671 of data sheet
    pub(super) fn alternate_function_select(&self) -> *mut u32 {
        const OFFSET: u32 = 0x420;
        (self.base() + OFFSET) as *mut u32
    }

    /// The memory address of the analog mode select (AMSEL) register for this port
    ///
    /// Page 687 of data sheet
    pub(super) fn analog_mode_select(&self) -> *mut u32 {
        const OFFSET: u32 = 0x52C;
        (self.base() + OFFSET) as *mut u32
    }

    /// The memory address of the commit (CR) register for this port
    ///
    /// Page 685 of data sheet
    pub(super) fn commit(&self) -> *mut u32 {
        const OFFSET: u32 = 0x524;
        (self.base() + OFFSET) as *mut u32
    }

    /// The memory address of the data (DATA) register for this port
    ///
    /// Page 662 of data sheet
    pub(super) fn data(&self, pins: &[Pin]) -> *mut u32 {
        // Extra guidance provided by
        // http://shukra.cedt.iisc.ernet.in/edwiki/EmSys:TM4C123GXL_GPIO_-_Read_Write_Data_Register
        // because the data sheet was a bit hard to understand when thinking about why
        // the C code I was referencing used an offset of 0x3FC
        let mut offset = 0;

        for pin in pins {
            let bit = (*pin as u32) + 2;
            offset |= 1 << bit;
        }

        (self.base() + offset) as *mut u32
    }

    /// The memory address of the digital enable (DEN) register for this port
    ///
    /// Page 682 of data sheet
    pub(super) fn digital_enable(&self) -> *mut u32 {
        const OFFSET: u32 = 0x51C;
        (self.base() + OFFSET) as *mut u32
    }

    /// The memory address of the direction (DIR) register for this port
    ///
    /// Page 663 of data sheet
    pub(super) fn direction(&self) -> *mut u32 {
        const OFFSET: u32 = 0x400;
        (self.base() + OFFSET) as *mut u32
    }

    /// The memory address of the lock (LOCK) register
    ///
    /// Page 684 of data sheet
    pub(super) fn lock(&self) -> *mut u32 {
        const OFFSET: u32 = 0x520;
        (self.base() + OFFSET) as *mut u32
    }

    /// The memory address of the port control (PCTL) register for this port
    ///
    /// Page 688 of data sheet
    pub(super) fn port_control(&self) -> *mut u32 {
        const OFFSET: u32 = 0x52C;
        (self.base() + OFFSET) as *mut u32
    }

    /// The memory address of the pull-down resistor select (PDR) register for this port
    /// Page 679 of data sheet
    pub(super) fn pull_down_select(&self) -> *mut u32 {
        const OFFSET: u32 = 0x514;
        (self.base() + OFFSET) as *mut u32
    }

    /// The memory address of the pull-up resistor select (PUR) register for this port
    /// Page 677 of data sheet
    pub(super) fn pull_up_select(&self) -> *mut u32 {
        const OFFSET: u32 = 0x510;
        (self.base() + OFFSET) as *mut u32
    }

    // TODO: examine page 690 (ADC) for applicability
    // Note to self: page 1351 of data sheet for PWM
    // Apparently also for ADC!
}

impl Port {
    /// The corresponding bit for this port in system's run-mode clock gate control (RCGC) register
    fn run_mode_clock_gate_control(&self) -> u32 {
        match self {
            Port::A => 0,
            Port::B => 1,
            Port::C => 2,
            Port::D => 3,
            Port::E => 4,
            Port::F => 5,
        }
    }
}

pub struct UsablePort {
    port: Port,
}

impl UsablePort {
    pub fn setup_readable_pins<const N: usize>(
        &self,
        pins: [Pin; N],
        options: ReadablePinOptions,
    ) -> ReadablePins<N> {
        setup_readable_pins(self.port, pins, options)
    }

    pub fn setup_writable_pins<const N: usize>(
        &self,
        pins: [Pin; N],
        options: WritablePinOptions,
    ) -> WritablePins<N> {
        setup_writable_pins(self.port, pins, options)
    }
}

pub fn setup_gpio_port(port: Port, _options: PortOptions) -> UsablePort {
    unsafe {
        memory::set_bits(
            registers::system::RCGCGPIO,
            &[port.run_mode_clock_gate_control() as u32],
        );
    }

    UsablePort { port }
}
