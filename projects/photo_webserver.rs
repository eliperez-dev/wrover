use esp_idf_svc::hal::peripherals::Peripherals;
// Import the raw C types
use esp_idf_svc::sys::camera::{
    esp_camera_init, esp_camera_fb_get, esp_camera_fb_return,
    camera_config_t,
    pixformat_t_PIXFORMAT_JPEG, framesize_t_FRAMESIZE_SVGA,
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
    // Use default() to create the struct, then modify fields.
    // This avoids needing to know the complex Union syntax.
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
    
    config.xclk_freq_hz = 20_000_000;
    config.ledc_timer = ledc_timer_t_LEDC_TIMER_0;
    config.ledc_channel = ledc_channel_t_LEDC_CHANNEL_0;
    config.pixel_format = pixformat_t_PIXFORMAT_JPEG;
    config.frame_size = framesize_t_FRAMESIZE_SVGA;
    config.jpeg_quality = 12;
    config.fb_count = 1;
    config.fb_location = 0;
    config.grab_mode = 0;

    unsafe {
        // __bindgen_anon_1 handles the SDA pin
        config.__bindgen_anon_1.pin_sccb_sda = pinout::SIOD;
        // __bindgen_anon_2 handles the SCL pin
        config.__bindgen_anon_2.pin_sccb_scl = pinout::SIOC;
    }

    unsafe {
        let err = esp_camera_init(&config);
        if err != ESP_OK {
            anyhow::bail!("Camera init failed with error: {}", err);
        }
    }

    // 3. START WEB SERVER
    let mut server = EspHttpServer::new(&Configuration::default())?;

    server.fn_handler("/", esp_idf_svc::http::Method::Get, |request| {
        unsafe {
            let fb = esp_camera_fb_get();
            if !fb.is_null() {
                let buf_ptr = std::ptr::addr_of!((*fb).buf);
                let len_ptr = std::ptr::addr_of!((*fb).len);
                
                let buf = buf_ptr.read_unaligned();
                let len = len_ptr.read_unaligned();

                let data = std::slice::from_raw_parts(buf, len as usize);
                
        let mut response = request.into_response(
            200, 
            Some("OK"), 
            &[("Content-Type", "image/jpeg")]
        )?;
                response.write_all(data)?;
                
                esp_camera_fb_return(fb);
            } else {
                request.into_status_response(500)?.write_all(b"Camera Capture Failed")?;
            }
        }
        Ok::<(), anyhow::Error>(())
    })?;

    println!("Server ready! Visit the IP in your browser.");

    loop {
        thread::sleep(Duration::from_secs(1));
    }
}