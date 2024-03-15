use std::error::Error;
use std::thread;
use std::time::Duration;
//use std::process::ExitCode;

use rppal::gpio::Gpio;

// Gpio uses BCM pin numbering. BCM GPIO 23 is tied to physical pin 16.
const GPIO_LED: u8 = 17;

fn main() -> Result<(), Box<dyn Error>> {
    // Retrieve the GPIO pin and configure it as an output.
    let mut pin = Gpio::new()?.get(GPIO_LED)?.into_output();

    loop {
        pin.toggle();
        thread::sleep(Duration::from_millis(500));
    }

    //if error {
    //    return ExitCode::from(1);
    //}

    // ExitCode::SUCCESS
}