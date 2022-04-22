#![no_std]
#![no_main]

use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics

use cortex_m_rt::entry;
use driver_and_task_library::{
    setup_board, Function, GPIOPortOptions, Pin, Port, Pull, ReadablePinOptions,
    WritablePinOptions, H, L,
};

#[entry]
fn main() -> ! {
    let board = setup_board();
    let port_f = board.setup_gpio_port(Port::F, GPIOPortOptions);

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

    let white = [H, H, H];
    let _black = [L, L, L];

    let red = [H, L, L];
    let yellow = [H, H, L];
    let green = [L, H, L];
    let cyan = [L, H, H];
    let blue = [L, L, H];
    let magenta = [H, L, H];

    let _rainbow = [red, yellow, green, cyan, blue, magenta];

    loop {
        match switches.read_all() {
            [L, L] => rgb_led.write_all(white),
            [L, H] => rgb_led.write_all(blue),
            [H, L] => rgb_led.write_all(red),
            [H, H] => rgb_led.write_all(green),
        }
    }
}
