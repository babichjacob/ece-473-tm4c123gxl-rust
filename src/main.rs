#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]

use core::{
    alloc::{GlobalAlloc, Layout},
    arch::asm,
    cell::UnsafeCell,
    ptr,
};

// Bump pointer allocator for *single* core systems
struct BumpPointerAlloc {
    head: UnsafeCell<usize>,
    end: usize,
}

unsafe impl Sync for BumpPointerAlloc {}

unsafe impl GlobalAlloc for BumpPointerAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // `interrupt::free` is a critical section that makes our allocator safe
        // to use from within interrupts
        interrupt::free(|_| {
            let head = self.head.get();
            let size = layout.size();
            let align = layout.align();
            let align_mask = !(align - 1);

            // move start up to the next alignment boundary
            let start = (*head + align - 1) & align_mask;

            if start + size > self.end {
                // a null pointer signal an Out Of Memory condition
                ptr::null_mut()
            } else {
                *head = start + size;
                start as *mut u8
            }
        })
    }

    unsafe fn dealloc(&self, _: *mut u8, _: Layout) {
        // this allocator never deallocates memory
    }
}

// Declaration of the global memory allocator
// NOTE the user must ensure that the memory region `[0x2000_0100, 0x2000_0200]`
// is not used by other parts of the program
#[global_allocator]
static HEAP: BumpPointerAlloc = BumpPointerAlloc {
    head: UnsafeCell::new(0x2000_0100),
    end: 0x2000_0200,
};

#[macro_use]
extern crate alloc;
use alloc::string::String;

use cortex_m::interrupt;
use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics

use cortex_m_rt::entry;
use driver_and_task_library::{
    setup_board, Function, Pin, Port, PortOptions, ReadablePinOptions, UsableBoard,
    WritablePinOptions, H, L, Pull,
};

const SYSCTL_RCGC1_R: *mut u32 = 0x400FE104 as *mut u32;
const SYSCTL_RCGC2_R: *mut u32 = 0x400FE108 as *mut u32;

const UART0_DR_R: *mut u32 = 0x4000C000 as *mut u32;
const UART0_FR_R: *mut u32 = 0x4000C018 as *mut u32;
const UART0_IBRD_R: *mut u32 = 0x4000C024 as *mut u32;
const UART0_FBRD_R: *mut u32 = 0x4000C028 as *mut u32;
const UART0_LCRH_R: *mut u32 = 0x4000C02C as *mut u32;
const UART0_CTL_R: *mut u32 = 0x4000C030 as *mut u32;

const GPIO_PORTA_AFSEL_R: *mut u32 = 0x40004420 as *mut u32;
const GPIO_PORTA_DEN_R: *mut u32 = 0x4000451C as *mut u32;

// page 219
/// 16 MHz
const SYSTEM_OSC_CLOCK_SPEED: u32 = 16_000_000;
// the MOSC is variable frequeny (5 MHz to 25 MHz)

// the XOSC can act as a real time clock as well!

// The internal system clock (SysClk), is derived from any of the above sources plus two others: the
// output of the main internal PLL and the precision internal oscillator divided by four (4 MHz ± 1%).
// The frequency of the PLL clock reference must be in the range of 5 MHz to 25 MHz (inclusive).
// Table 5-3 on page 220 shows how the various clock sources can be used in a system

/// UART0 Clock Gating Control
const SYSCTL_RCGC1_UART0: u32 = 0x00000001;
/// port A Clock Gating Control
const SYSCTL_RCGC2_GPIOA: u32 = 0x00000001;
/// UART Enable
const UART_CTL_UARTEN: u32 = 0x00000001;
/// 8 bit word length
const UART_LCRH_WLEN_8: u32 = 0x00000060;
/// UART Enable FIFOs
const UART_LCRH_FEN: u32 = 0x00000010;
/// UART Transmit FIFO Full
const UART_FR_TXFF: u32 = 0x00000020;
/// UART Receive FIFO Empty
const UART_FR_RXFE: u32 = 0x00000010;
/// Pins 0 and 1
const PINS_0_AND_1: u32 = 0b0000_0011;

fn uart0_init(board: &mut UsableBoard) {
    unsafe {
        // activate UART0
        ptr::write_volatile(
            SYSCTL_RCGC1_R,
            ptr::read_volatile(SYSCTL_RCGC1_R) | SYSCTL_RCGC1_UART0,
        );

        // activate port A
        // ptr::write_volatile(SYSCTL_RCGC2_R, ptr::read_volatile(SYSCTL_RCGC2_R) | SYSCTL_RCGC2_GPIOA);
        // ^ commented in favor of v
        board.setup_gpio_port(Port::A, PortOptions);

        // disable UART while setting it up
        ptr::write_volatile(
            UART0_CTL_R,
            ptr::read_volatile(UART0_CTL_R) & !UART_CTL_UARTEN,
        );

        // ignore: // IBRD = int(50,000,000 / (16 * 115,200)) = int(27.1267)
        // IBRD = int(16,000,000 / (16 * 115,200)) = int(8.680)
        // ptr::write_volatile(UART0_IBRD_R, 8);
        ptr::write_volatile(UART0_IBRD_R, 8);

        // ignore: // FBRD = int(0.1267 * 64 + 0.5) = 8
        // FBRD = round(0.5104 * 64 ) = 33 --- that ain't the number you wrote but ok
        // ptr::write_volatile(UART0_FBRD_R, 44);
        ptr::write_volatile(UART0_FBRD_R, 44);

        // 8 bit word length (no parity bits, one stop bit, FIFOs)
        // ptr::write_volatile(UART0_LCRH_R, UART_LCRH_WLEN_8|UART_LCRH_FEN);
        // 8 bit word length (no parity bits, one stop bit, no FIFOs)
        ptr::write_volatile(UART0_LCRH_R, UART_LCRH_WLEN_8 & !UART_LCRH_FEN);

        // enable UART since it's been set up
        ptr::write_volatile(
            UART0_CTL_R,
            ptr::read_volatile(UART0_CTL_R) | UART_CTL_UARTEN,
        );

        // enable alt funct on PA1-0
        ptr::write_volatile(
            GPIO_PORTA_AFSEL_R,
            ptr::read_volatile(GPIO_PORTA_AFSEL_R) | PINS_0_AND_1,
        );
        // enable digital I/O on PA1-0
        ptr::write_volatile(
            GPIO_PORTA_DEN_R,
            ptr::read_volatile(GPIO_PORTA_AFSEL_R) | PINS_0_AND_1,
        );
    }
}

fn uart0_out_char(c: u8) {
    loop {
        let fr = unsafe { ptr::read_volatile(UART0_FR_R) };

        if (fr & UART_FR_TXFF) == 0 {
            break;
        }
    }

    unsafe {
        ptr::write_volatile(UART0_DR_R, c as u32);
    }
}

fn uart0_out_string(s: &str) {
    for c in s.bytes() {
        uart0_out_char(c);
    }
}

const WHITE: [bool; 3] = [H, H, H];
const BLACK: [bool; 3] = [L, L, L];

const RED: [bool; 3] = [H, L, L];
const YELLOW: [bool; 3] = [H, H, L];
const GREEN: [bool; 3] = [L, H, L];
const CYAN: [bool; 3] = [L, H, H];
const BLUE: [bool; 3] = [L, L, H];
const MAGENTA: [bool; 3] = [H, L, H];

#[entry]
fn main() -> ! {
    let mut board = setup_board();
    let mut port_a = board.setup_gpio_port(Port::A, PortOptions);
    let mut port_f = board.setup_gpio_port(Port::F, PortOptions);

    uart0_init(&mut board);

    // WIP: page 682
    port_a.setup_writable_pins(
        [Pin::One],
        WritablePinOptions {
            function: Function::UART,
        },
    );

    uart0_out_string("Hi, this is after uart setup_writable_pins\r\n\r\n");
    // TODO: finish this
    // port_a.setup_readable_pins([Pin::Zero], WritablePinOptions {
    //         function: Function::UART,
    //     });
    //     uart0_out_string("Hi, this is after uart setup_readable_pins\r\n\r\n");

    let switches = port_f.setup_readable_pins(
        [Pin::Zero, Pin::Four],
        ReadablePinOptions {
            function: Function::Digital,
            pull: Pull::Up,
        },
    );
    let [_sw1, _sw2] = switches.pins();

    let mut rgb_led = port_f.setup_writable_pins(
        [Pin::One, Pin::Three, Pin::Two],
        WritablePinOptions {
            function: Function::Digital,
        },
    );


    let rainbow = [RED, YELLOW, GREEN, CYAN, BLUE, MAGENTA];

    // TODO: WIP: debugging
    // let s = format!("Rainbow: {:?}\r\n", rainbow);
    let s = String::from("\r\ntesting a static string!!!\r\n\r\n\r\n");
    // let s = format!("Format");
    // let s: String = rainbow.into();
    uart0_out_string(&s);

    loop {
        uart0_out_string("Hi the program is still running down here!\r\n");
        match switches.read_all() {
            [L, L] => rgb_led.write_all(WHITE),
            [L, H] => rgb_led.write_all(BLUE),
            [H, L] => rgb_led.write_all(RED),
            [H, H] => rgb_led.write_all(BLACK),
        }

        // uart0_out_string(&format!("The switches read {:?}", switches.read_all()));

        for _ in 0..1000000 {
            unsafe {
                asm!("nop");
            }
        }
    }
}
