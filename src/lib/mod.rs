#![no_std]

mod board;
mod gpio;
mod memory;
mod registers;

pub use board::*;
pub use gpio::pins::*;
pub use gpio::ports::*;

pub const H: bool = true;
pub const L: bool = false;
