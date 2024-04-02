use std::error::Error;
use std::thread;
use std::time::Duration;

use rppal::gpio::{Gpio, Level, OutputPin, Trigger};
use soloud::*;

// Gpio uses BCM pin numbering.
const GPIO_GREEN_LED: u8 = 17; // Pin #11
const GPIO_RED_LED: u8 = 27; // Pin #13
const GPIO_WHITE_BUTTON: u8 = 2; // Pin #3
const HOOK_UP: u8 = 3; // Pin #5
const MOTOR_ENABLE: u8 = 16;
const MOTOR_1: u8 = 20;
const MOTOR_2: u8 = 21;

fn main() -> Result<(), Box<dyn Error>> {
    let mut motor_enable = Gpio::new()?.get(MOTOR_ENABLE)?.into_output();
    let mut motor_1 = Gpio::new()?.get(MOTOR_1)?.into_output();
    let mut motor_2 = Gpio::new()?.get(MOTOR_2)?.into_output();

    let green = Gpio::new()?.get(GPIO_GREEN_LED)?.into_output();
    let mut red = Gpio::new()?.get(GPIO_RED_LED)?.into_output();
    red.write(Level::Low);

    let mut white_button = Gpio::new()?.get(GPIO_WHITE_BUTTON)?.into_input_pullup();
    let mut hookup = Gpio::new()?.get(HOOK_UP)?.into_input_pullup();

    let sl = Soloud::default().expect("Could not get Soloud");
    let mut speech = audio::Speech::default();
    speech.set_text("Yes?")?;

    thread::spawn(move || heartbeat(green));

    // setup initial value
    red.write(hookup.read());

    hookup.set_async_interrupt(Trigger::Both, move |level| {
        red.write(level);
        match level {
            Level::Low => {}
            Level::High => {
                thread::sleep(Duration::from_millis(700));
                println!("Yes?");
                sl.play(&speech);
            }
        }
    })?;

    white_button.set_async_interrupt(Trigger::FallingEdge, move |_| {
        ring(4, &mut motor_enable, &mut motor_1, &mut motor_2);
    })?;

    thread::sleep(Duration::from_secs(10000));

    Ok(())
}

fn ring(
    number: u32,
    motor_enable: &mut OutputPin,
    motor_1: &mut OutputPin,
    motor_2: &mut OutputPin,
) {
    motor_enable.write(Level::High);
    println!("Starting ringing");

    for _ in 0..number {
        motor_1.write(Level::High);
        motor_2.write(Level::Low);

        thread::sleep(Duration::from_millis(15));

        motor_1.write(Level::Low);
        motor_2.write(Level::Low);

        thread::sleep(Duration::from_millis(5));

        motor_1.write(Level::Low);
        motor_2.write(Level::High);

        thread::sleep(Duration::from_millis(15));

        motor_1.write(Level::Low);
        motor_2.write(Level::Low);

        thread::sleep(Duration::from_millis(5));
    }

    println!("Done ringing");
    motor_1.write(Level::Low);
    motor_2.write(Level::Low);
    motor_enable.write(Level::Low);
}

fn heartbeat(mut pin: OutputPin) {
    loop {
        pin.toggle();
        thread::sleep(Duration::from_millis(500));
    }
}
