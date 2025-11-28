use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::gpio::*;
use std::thread;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    // Basic ESP-IDF setup
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    // 1. Take peripherals
    let peripherals = Peripherals::take().unwrap();

    // 2. Configure LED Pin
    // ESP32-C3 pins - adjust based on your board's LED connections
    let mut led = PinDriver::output(peripherals.pins.gpio26)?;


    // 3. Blink Loop
    loop {
        // Turn LED On
        led.set_high()?; 
        std::thread::sleep(Duration::from_millis(1000));
        led.set_low()?;
        std::thread::sleep(Duration::from_millis(1000));

    };
}