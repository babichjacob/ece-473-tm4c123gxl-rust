#![no_std]
#![no_main]

use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics

use cortex_m_rt::entry;
use test_cortex_m4_rust::{
    setup_board, Bit, Function, Port, PortSetup, ReadablePinSetup, WritablePinSetup, H, L,
};

#[entry]
fn main() -> ! {
    let board = setup_board();
    let port_f = board.setup_gpio_port(Port::F, PortSetup);

    let switches = port_f.setup_readable_pins(&[Bit::Zero, Bit::Four], ReadablePinSetup {
        function: Function::Digital,
        pull_up: Some(true),
    });
    let [sw1, sw2] = switches.pins();

    let mut rgb_led = port_f.setup_writable_pins(
        &[Bit::One, Bit::Three, Bit::Two],
        WritablePinSetup {
            function: Function::Digital,
        },
    );

    let white = [H, H, H];
    let black = [L, L, L];

    let red = [H, L, L];
    let yellow = [H, H, L];
    let green = [L, H, L];
    let cyan = [L, H, H];
    let blue = [L, L, H];
    let magenta = [H, L, H];

    let rainbow = [red, yellow, green, cyan, blue, magenta];

    loop {
        match switches.read_all() {
            [L, L] => rgb_led.write_all(white),
            [L, H] => rgb_led.write_all(red),
            [H, L] => rgb_led.write_all(green),
            [H, H] => rgb_led.write_all(blue),
        }
    }
}

// TODO: implement an extremely simple example of the task system (using a timer trigger) that is a traffic light (green -> yellow -> red -> green...)
