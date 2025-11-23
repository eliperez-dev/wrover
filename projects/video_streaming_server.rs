use esp_idf_svc::hal::peripherals::Peripherals;

#[allow(unused_imports)]
use esp_idf_svc::sys::camera::{
    esp_camera_init, esp_camera_fb_get, esp_camera_fb_return,
    camera_config_t,
    // The specific types for the Union fix
    camera_config_t__bindgen_ty_1, camera_config_t__bindgen_ty_2,
    
    // Pixel Formats
    pixformat_t_PIXFORMAT_JPEG,
    
    // Frame Sizes (Resolutions) - I added the common ones here for you
    framesize_t_FRAMESIZE_UXGA, // 1600x1200 (High Res, Slow)
    framesize_t_FRAMESIZE_SXGA, // 1280x1024
    framesize_t_FRAMESIZE_XGA,  // 1024x768
    framesize_t_FRAMESIZE_SVGA, // 800x600  (Medium, Standard)
    framesize_t_FRAMESIZE_VGA,  // 640x480  (Standard Webcam)
    framesize_t_FRAMESIZE_QVGA, // 320x240  (Fast Video)
    
    // Timer/Channel settings
    ledc_channel_t_LEDC_CHANNEL_0, ledc_timer_t_LEDC_TIMER_0,
    esp_err_t, ESP_OK
};
use esp_idf_svc::wifi::{BlockingWifi, EspWifi};
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::http::server::{Configuration, EspHttpServer};
use esp_idf_svc::io::Write;
use std::thread;
use std::time::Duration;

// FREENOVE WROVER-E PINOUT
// (Matches standard WROVER-KIT definition)
mod pinout {
    pub const PWDN: i32 = -1;
    pub const RESET: i32 = -1;
    pub const XCLK: i32 = 21;
    pub const SIOD: i32 = 26;
    pub const SIOC: i32 = 27;
    pub const Y9: i32 = 35;
    pub const Y8: i32 = 34;
    pub const Y7: i32 = 39;
    pub const Y6: i32 = 36;
    pub const Y5: i32 = 19;
    pub const Y4: i32 = 18;
    pub const Y3: i32 = 5;
    pub const Y2: i32 = 4;
    pub const VSYNC: i32 = 25;
    pub const HREF: i32 = 23;
    pub const PCLK: i32 = 22;
}

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
    let mut config = camera_config_t::default();

    config.pin_pwdn = pinout::PWDN;
    config.pin_reset = pinout::RESET;
    config.pin_xclk = pinout::XCLK;
    config.pin_d7 = pinout::Y9;
    config.pin_d6 = pinout::Y8;
    config.pin_d5 = pinout::Y7;
    config.pin_d4 = pinout::Y6;
    config.pin_d3 = pinout::Y5;
    config.pin_d2 = pinout::Y4;
    config.pin_d1 = pinout::Y3;
    config.pin_d0 = pinout::Y2;
    config.pin_vsync = pinout::VSYNC;
    config.pin_href = pinout::HREF;
    config.pin_pclk = pinout::PCLK;
    
    // --- SETTINGS YOU CAN CHANGE ---
    config.xclk_freq_hz = 20_000_000; // 20MHz. Try 10MHz if wifi is unstable.
    config.pixel_format = pixformat_t_PIXFORMAT_JPEG;
    
    // RESOLUTION: Change this to QVGA for speed, UXGA for detail
    config.frame_size = framesize_t_FRAMESIZE_SVGA; // 800x600
    
    config.jpeg_quality = 12; // 10-63 (Lower is better quality)
    
    config.fb_count = 2; // Double Buffering (Faster video, requires PSRAM)
    config.fb_location = 0; // DRAM/PSRAM
    config.grab_mode = 0;   // When Empty

    config.ledc_timer = ledc_timer_t_LEDC_TIMER_0;
    config.ledc_channel = ledc_channel_t_LEDC_CHANNEL_0;

    // Union Fix for Driver v3.x+
    config.__bindgen_anon_1.pin_sccb_sda = pinout::SIOD;
    config.__bindgen_anon_2.pin_sccb_scl = pinout::SIOC;

    unsafe {
        let err = esp_camera_init(&config);
        if err != ESP_OK {
            anyhow::bail!("Camera init failed with error: {}", err);
        }
    }

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