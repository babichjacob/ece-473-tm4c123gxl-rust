#![no_std]

pub struct Board;

impl Board {
    pub fn setup_gpio_port(port: Port, options: PortSetup) -> PortIO {
        PortIO
    }
}

pub enum Port {
    A,
    F,
}

pub struct PortSetup;

pub struct PortIO;

impl PortIO {
    pub fn setup_pin(pin: Pin, options: PinSetup) -> PinIO {
        PinIO
    }
}

pub enum Pin {
    Zero = 0,
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    Seven = 7,
}

pub struct PinSetup;

pub struct PinIO;

impl PinIO {
    pub fn clear(&mut self) {
        todo!();
    }
    pub fn set(&mut self) {
        todo!();
    }
    pub fn toggle(&mut self) {
        todo!();
    }
}


pub fn setup_board() -> Board {
    Board
}
