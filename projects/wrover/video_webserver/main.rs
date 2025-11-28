use esp_idf_svc::hal::peripherals::Peripherals;


use esp_idf_svc::sys::camera::{
    esp_camera_fb_get, esp_camera_fb_return,

};
use esp_idf_svc::wifi::{BlockingWifi, EspWifi};
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::http::server::{Configuration, EspHttpServer};
use esp_idf_svc::io::Write;
use std::thread;
use std::time::Duration; 

mod ov3660; 

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    // 1. SETUP WIFI
    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs))?,
        sys_loop,
    )?;

    wifi.set_configuration(&esp_idf_svc::wifi::Configuration::Client(
        esp_idf_svc::wifi::ClientConfiguration {
            // !!! UPDATE THESE !!!
            ssid: "Verizon-5G-Home-26A9".try_into().unwrap(),
            password: "dust-cute4-fay".try_into().unwrap(),
            ..Default::default()
        },
    ))?;

    wifi.start()?;
    wifi.connect()?;
    wifi.wait_netif_up()?;

    println!("Wifi connected! IP: {:?}", wifi.wifi().sta_netif().get_ip_info()?.ip);

    // 2. SETUP CAMERA
    let _ = ov3660::start_ov3660(ov3660::OV3660Config::high_quality());

    // 3. START MJPEG STREAM SERVER
    let mut server = EspHttpServer::new(&Configuration::default())?;

    server.fn_handler("/", esp_idf_svc::http::Method::Get, |request| {
        // Send Multipart Header
        let mut response = request.into_response(
            200, 
            Some("OK"), 
            &[("Content-Type", "multipart/x-mixed-replace; boundary=123456789000000000000987654321")]
        )?;

        println!("Client connected to stream.");

        loop {
            unsafe {
                let fb = esp_camera_fb_get();
                if !fb.is_null() {
                    // Fix: Unaligned Read for Timestamp field (prevents panic)
                    let buf_ptr = std::ptr::addr_of!((*fb).buf);
                    let len_ptr = std::ptr::addr_of!((*fb).len);
                    let buf = buf_ptr.read_unaligned();
                    let len = len_ptr.read_unaligned();
                    
                    let data = std::slice::from_raw_parts(buf, len as usize);

                    // MJPEG Frame Header
                    let header = format!(
                        "\r\n--123456789000000000000987654321\r\nContent-Type: image/jpeg\r\nContent-Length: {}\r\n\r\n",
                        len
                    );

                    // Send Header
                    if response.write_all(header.as_bytes()).is_err() {
                        esp_camera_fb_return(fb);
                        println!("Client disconnected.");
                        break;
                    }

                    // Send Image
                    if response.write_all(data).is_err() {
                        esp_camera_fb_return(fb);
                        break;
                    }

                    esp_camera_fb_return(fb);
                }
            }
            // Small delay to let the CPU breathe
            // Remove this for maximum FPS, but keep it if the board gets too hot
            thread::sleep(Duration::from_millis(20)); 
        }
        
        Ok::<(), anyhow::Error>(())
    })?;

    println!("Server ready!");

    // Keep main thread alive
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}