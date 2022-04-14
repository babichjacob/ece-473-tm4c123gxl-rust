use crate::gpio::ports::{setup_gpio_port, Port, PortOptions, UsablePort};

pub struct UsableBoard;
impl UsableBoard {
    pub fn setup_gpio_port(&self, port: Port, options: PortOptions) -> UsablePort {
        setup_gpio_port(port, options)
    }

    // TODO: check page 704 for timers
    // TODO: impl Drop trait so that tasks all run before the main function ends?
    // TODO: examine page 670 for when (if) I do interrupts
}

pub fn setup_board() -> UsableBoard {
    UsableBoard
}
