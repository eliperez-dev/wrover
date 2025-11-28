use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::gpio::PinDriver;
use std::thread;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    // Basic ESP-IDF setup
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    // 1. Take peripherals
    let peripherals = Peripherals::take().unwrap();

    // 2. Configure LED Pin
    // On ESP32-CAM, gpio4 is the flashlight. 
    // On standard ESP32 DevKits, gpio2 is usually the blue onboard LED.
    let mut led = PinDriver::output(peripherals.pins.gpio4)?;

    println!("Blinky started!");

    // 3. Blink Loop
    loop {
        // Turn LED On
        led.set_high()?;
        println!("LED ON");
        thread::sleep(Duration::from_millis(100));

        // Turn LED Off
        led.set_low()?;
        println!("LED OFF");
        thread::sleep(Duration::from_millis(100));
    }
}