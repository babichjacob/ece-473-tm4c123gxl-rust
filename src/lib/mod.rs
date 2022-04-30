#![no_std]
#![feature(alloc_error_handler)]

mod board;
mod gpio;
mod memory;
mod uart;
mod utils;

pub use board::*;
pub use gpio::pins::*;
pub use gpio::ports::{Port as GPIOPort, PortOptions as GPIOPortOptions};
pub use uart::{Port as UARTPort, PortOptions as UARTPortOptions, WordLength};

pub const H: bool = true;
pub const L: bool = false;

extern crate alloc;
use alloc::string::ToString;
use core::alloc::{GlobalAlloc, Layout};
use core::panic::PanicInfo;
use core::ptr;

const BLACK: [bool; 3] = [L, L, L];

const RED: [bool; 3] = [H, L, L];
const YELLOW: [bool; 3] = [H, H, L];
const CYAN: [bool; 3] = [L, H, H];

#[panic_handler]
fn panic(panic_info: &PanicInfo) -> ! {
    let mut board = setup_board();

    let mut port_f = board.setup_gpio_port(GPIOPort::F);

    let mut rgb_led = port_f.setup_writable_pins(
        [Pin::One, Pin::Three, Pin::Two],
        WritablePinOptions {
            function: Function::Digital,
        },
    );

    // Set the LED to red in case setting up UART causes the system to hang
    // and the loop where we flash red / cyan isn't reached
    // (but there's no reason that should happen...)
    rgb_led.write_all(RED);

    let mut port_a = board.setup_gpio_port(GPIOPort::A);
    let [_uart_0_rx] = port_a
        .setup_readable_pins(
            [Pin::Zero],
            ReadablePinOptions {
                function: Function::UART,
                pull: Pull::Neither,
            },
        )
        .pins();
    let [mut uart_0_tx] = port_a
        .setup_writable_pins(
            [Pin::One],
            WritablePinOptions {
                function: Function::UART,
            },
        )
        .pins();
    let mut uart_0 = board.setup_uart_port(
        UARTPort::Zero,
        UARTPortOptions {
            baud_rate: 115_200,
            fifos: true,
            word_length: WordLength::Eight,
        },
    );

    // https://gist.github.com/fnky/458719343aabd01cfb17a3a4f7296797
    uart_0.write_line(&mut uart_0_tx, "\x1b[31m");
    uart_0.write_line(&mut uart_0_tx, &panic_info.to_string());
    uart_0.write_line(&mut uart_0_tx, "\x1b[0m");

    let pattern = [RED, BLACK, CYAN, BLACK];

    loop {
        for color in pattern {
            rgb_led.write_all(color);
            board.no_ops(1_000_000);
        }
    }
}

struct BumpPointerAlloc;
static mut HEAP: [u8; 0x1000] = [0; 0x1000];
static mut USED: usize = 0;

#[global_allocator]
static ALLOCATOR: BumpPointerAlloc = BumpPointerAlloc;

unsafe impl GlobalAlloc for BumpPointerAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        
        if USED + size > HEAP.len() {
            ptr::null_mut()
        } else {
            let pointer = &mut HEAP[USED] as *mut u8;
            USED += size;

            pointer
        }
    }

    unsafe fn dealloc(&self, _: *mut u8, _: Layout) {}
}

#[alloc_error_handler]
fn alloc_error(_cause: Layout) -> ! {
    let mut board = setup_board();

    let mut port_f = board.setup_gpio_port(GPIOPort::F);

    let mut rgb_led = port_f.setup_writable_pins(
        [Pin::One, Pin::Three, Pin::Two],
        WritablePinOptions {
            function: Function::Digital,
        },
    );

    let pattern = [YELLOW, BLACK, RED, BLACK];

    loop {
        for color in pattern {
            rgb_led.write_all(color);
            board.no_ops(1_000_000);
        }
    }
}
