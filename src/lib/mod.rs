#![no_std]

mod board;
mod gpio;
mod memory;
mod registers;

pub use board::setup_board;
pub use gpio::pins::{
    Function, Pin, ReadablePin, ReadablePinOptions, ReadablePins, WritablePin, WritablePinOptions,
    WritablePins,
};
pub use gpio::ports::{Port, PortOptions, UsablePort};

pub const H: bool = true;
pub const L: bool = false;
