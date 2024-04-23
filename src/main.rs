use std::error::Error;
use std::path::PathBuf;
use std::time::Duration;
use std::{env, io, thread};

use rppal::gpio::{Gpio, InputPin, Level, OutputPin, Trigger};
use service_manager::{
    ServiceInstallCtx, ServiceLabel, ServiceManager, ServiceStartCtx, ServiceStopCtx,
    ServiceUninstallCtx,
};
use soloud::*;

// Gpio uses BCM pin numbering.
const GPIO_WHITE_BUTTON: u8 = 2;
// Pin #3
const HOOK_UP: u8 = 3;
// Pin #5
const MOTOR_ENABLE: u8 = 16;
const MOTOR_1: u8 = 20;
const MOTOR_2: u8 = 21;

const SERVICE_NAME: &str = "net.mackenzie-serres.ringr";

fn main() -> Result<(), io::Error> {
    let service_label: ServiceLabel = SERVICE_NAME.parse().unwrap();

    let args: Vec<_> = env::args().collect();
    match args.get(1).map(|s| s.as_str()) {
        None => run().unwrap(),
        Some("install") => install_service(&service_label, &args[0])?,
        Some("uninstall") => uninstall_service(&service_label)?,
        _ => eprintln!("Invalid argument(s): '{}'", &args[1..].join(", ")),
    }

    Ok(())
}

fn run() -> Result<(), Box<dyn Error>> {
    let mut motor_enable = Gpio::new()?.get(MOTOR_ENABLE)?.into_output();
    let mut motor_1 = Gpio::new()?.get(MOTOR_1)?.into_output();
    let mut motor_2 = Gpio::new()?.get(MOTOR_2)?.into_output();

    let mut white_button = Gpio::new()?.get(GPIO_WHITE_BUTTON)?.into_input_pullup();
    let mut hookup = Gpio::new()?.get(HOOK_UP)?.into_input_pullup();

    let sl = Soloud::default().expect("Could not get Soloud");
    let mut speech = audio::Speech::default();
    speech.set_text("Yes?")?;

    // initial hangup switch position
    println!("Hookup: {}", hookup.read());

    // startup ring
    ring(8, &mut motor_enable, &mut motor_1, &mut motor_2);

    white_button.set_async_interrupt(Trigger::FallingEdge, move |_| {
        ring(4, &mut motor_enable, &mut motor_1, &mut motor_2);
    })?;

    loop {
        match debounce(&mut hookup, Trigger::FallingEdge)? {
            Level::High => println!("On the hook"),
            Level::Low => {
                print!("Off the hook");
                thread::sleep(Duration::from_millis(700));
                println!("Yes?");
                sl.play(&speech);

                let _ = debounce(&mut hookup, Trigger::RisingEdge)?;
                println!("Back on the hook");
            }
        }
    }
}

fn debounce(pin: &mut InputPin, trigger: Trigger) -> Result<Level, Box<dyn Error>> {
    pin.set_interrupt(trigger)?;

    loop {
        if let Ok(Some(lev)) = pin.poll_interrupt(false, None) {
            for _count in 0..5 {
                thread::sleep(Duration::from_millis(100));
                if pin.read() != lev {
                    continue;
                }
            }
            return Ok(lev);
        }
        println!("Bounce!");
    }
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

fn get_service_manager() -> Result<Box<dyn ServiceManager>, io::Error> {
    // Get generic service by detecting what is available on the platform
    let manager = <dyn ServiceManager>::native()
        .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "Could not create ServiceManager"))?;

    Ok(manager)
}

// This will install the binary as a user level service and then start it
fn install_service(service_name: &ServiceLabel, path_to_exec: &str) -> Result<(), io::Error> {
    let manager = get_service_manager()?;
    let exec_path = PathBuf::from(path_to_exec).canonicalize()?;
    // Run from dir where exec is for now, so it should find the config file in ancestors path
    let exec_dir = exec_path
        .parent()
        .ok_or(io::Error::new(
            io::ErrorKind::NotFound,
            "Could not get exec dir",
        ))?
        .to_path_buf();

    // Install our service using the underlying service management platform
    manager.install(ServiceInstallCtx {
        label: service_name.clone(),
        program: exec_path,
        args: vec![],
        contents: None, // Optional String for system-specific service content.
        username: None, // Optional String for alternative user to run service.
        working_directory: Some(exec_dir),
        environment: None, // Optional list of environment variables to supply the service process.
    })?;

    // Start our service using the underlying service management platform
    manager.start(ServiceStartCtx {
        label: service_name.clone(),
    })?;

    println!("'service '{service_name}' ('{path_to_exec}') installed and started");

    Ok(())
}

// this will stop any running instance of the service, then uninstall it
fn uninstall_service(service_name: &ServiceLabel) -> Result<(), io::Error> {
    let manager = get_service_manager()?;

    // Stop our service using the underlying service management platform
    manager.stop(ServiceStopCtx {
        label: service_name.clone(),
    })?;

    println!(
        "service '{}' stopped. Waiting for 10s before uninstalling",
        service_name
    );
    thread::sleep(Duration::from_secs(10));

    // Uninstall our service using the underlying service management platform
    manager.uninstall(ServiceUninstallCtx {
        label: service_name.clone(),
    })?;

    println!("service '{}' uninstalled", service_name);

    Ok(())
}
