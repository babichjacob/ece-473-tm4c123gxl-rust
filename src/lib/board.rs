//! Data sheet: https://www.ti.com/lit/ds/spms376e/spms376e.pdf

use core::arch::asm;

use crate::gpio::ports::{
    setup_port as setup_gpio_port, Port as GPIOPort, UsablePort as UsableGPIOPort,
};
use crate::uart::{
    setup_port as setup_uart_port, Port as UARTPort, PortOptions as UARTPortOptions,
    UsablePort as UsableUARTPort,
};

/// The board
///
/// Houses memory addresses of registers
#[derive(Clone, Copy)]
pub struct Board;

impl Board {
    /// Page 231 of data sheet
    const fn base(&self) -> u32 {
        0x400F_E000
    }

    /// The memory address of the GPIO Run mode clock gating control (RCGCGPIO) register for GPIO ports
    ///
    /// Page 340 of data sheet
    pub(crate) const fn gpio_run_mode_clock_gating_control(&self) -> *mut u32 {
        const OFFSET: u32 = 0x608;
        (self.base() + OFFSET) as *mut u32
    }

    /// The memory address of the TODO
    ///
    /// Page TODO of data sheet
    pub(crate) const fn run_mode_clock_gating_control_1(&self) -> *mut u32 {
        const OFFSET: u32 = 0x104;
        (self.base() + OFFSET) as *mut u32
    }
}

/// A setup version of the board that GPIO and UART ports (TODO: say more features when I make those) can be set up on
pub struct UsableBoard {
    board: Board,
}

impl UsableBoard {
    pub(crate) fn no_op(&self) {
        unsafe {
            asm!("nop");
        }
    }

    pub(crate) fn no_ops(&self, n: u32) {
        for _ in 0..n {
            self.no_op();
        }
    }
}

impl UsableBoard {
    pub fn setup_gpio_port(&mut self, port: GPIOPort) -> UsableGPIOPort {
        setup_gpio_port(self.board, port)
    }

    pub fn setup_uart_port(&mut self, port: UARTPort, options: UARTPortOptions) -> UsableUARTPort {
        setup_uart_port(self.board, port, options, &|n| self.no_ops(n))
    }
}

/// Start using the driver and task library by setting up the TM4C123GXL board
pub fn setup_board() -> UsableBoard {
    UsableBoard { board: Board }
}
