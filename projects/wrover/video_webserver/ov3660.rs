
use esp_idf_svc::sys::camera::{
    esp_camera_init,
    camera_config_t,

    pixformat_t_PIXFORMAT_JPEG,
    pixformat_t_PIXFORMAT_RGB888,

    framesize_t_FRAMESIZE_SVGA, 
    framesize_t_FRAMESIZE_QVGA,
    framesize_t_FRAMESIZE_QXGA,

    ledc_channel_t_LEDC_CHANNEL_0,
    ledc_timer_t_LEDC_TIMER_0,
    ESP_OK
};
use esp_idf_svc::sys::camera::*; // Import all 


#[derive(Clone, Copy)]
pub enum OV3660Format {
    /// Compressed JPEG for Streaming / Webcam. 0-63, lower is higher quality
    JPEG { quality: u8 },
    /// 24-bit True Color, High RAM Usage (Don't use for WiFi streaming)
    RGB888,
    /// 8-bit Grayscale (Good for AI/CV)
    Grayscale,
}

#[derive(Clone, Copy)]
pub enum OV3660Resolution {
    /// QXGA 2048x1536 (Use with FB_COUNT=1)
    HighRes,
    /// SVGA 800x600 (Standard)
    MedRes,
    /// QVGA 320x240 (Fast)
    LowRes,
}

#[derive(Clone, Copy)]
pub enum OV3660ClockSpeed {
    /// 20MHz (Standard)
    High,
    /// 10MHz (Use if WiFi is unstable)
    Low,
}

pub struct OV3660Config {
    pub format: OV3660Format,
    pub camera_resolution: OV3660Resolution,
    pub double_buffered: bool,
    pub clock_speed: OV3660ClockSpeed,
}

impl OV3660Config {
    pub fn new(format: OV3660Format, camera_resolution: OV3660Resolution, double_buffered: bool, clock_speed: OV3660ClockSpeed) -> Self {
        Self {
            format,
            camera_resolution,
            double_buffered,
            clock_speed,
        }
    }

    /// Best for getting 25+ FPS
    pub fn fast_streaming() -> Self {
        Self::new(
            OV3660Format::JPEG { quality: 12 }, 
            OV3660Resolution::LowRes, 
            true, 
            OV3660ClockSpeed::High
        )
    }

    /// Best for clear static images
    pub fn high_quality() -> Self {
        Self::new(
            OV3660Format::JPEG { quality: 10 }, // 0 is risky, 10 is safe high-quality
            OV3660Resolution::HighRes, 
            false, // HighRes usually requires single buffer due to RAM limits
            OV3660ClockSpeed::Low // Slower clock for better signal stability on large frames
        )
    }

    /// Good balance for general use
    pub fn balanced() -> Self {
        Self::new(
            OV3660Format::JPEG { quality: 12 }, 
            OV3660Resolution::MedRes, 
            true, 
            OV3660ClockSpeed::High
        )
    }
}

pub fn start_ov3660(user_config: OV3660Config) -> anyhow::Result<()> {
    let mut camera_config = camera_config_t::default();

    // Map the pins from your pinout module
    camera_config.pin_pwdn = pinout::PWDN;
    camera_config.pin_reset = pinout::RESET;
    camera_config.pin_xclk = pinout::XCLK;
    camera_config.pin_d7 = pinout::Y9;
    camera_config.pin_d6 = pinout::Y8;
    camera_config.pin_d5 = pinout::Y7;
    camera_config.pin_d4 = pinout::Y6;
    camera_config.pin_d3 = pinout::Y5;
    camera_config.pin_d2 = pinout::Y4;
    camera_config.pin_d1 = pinout::Y3;
    camera_config.pin_d0 = pinout::Y2;
    camera_config.pin_vsync = pinout::VSYNC;
    camera_config.pin_href = pinout::HREF;
    camera_config.pin_pclk = pinout::PCLK;

    // Logic Configuration
    camera_config.xclk_freq_hz = match user_config.clock_speed {
        OV3660ClockSpeed::High => 20_000_000,
        OV3660ClockSpeed::Low => 10_000_000,
    };

    camera_config.pixel_format = match user_config.format {
        OV3660Format::JPEG { .. } => pixformat_t_PIXFORMAT_JPEG,
        OV3660Format::RGB888 => pixformat_t_PIXFORMAT_RGB888,
        OV3660Format::Grayscale => pixformat_t_PIXFORMAT_GRAYSCALE,
    };

    camera_config.frame_size = match user_config.camera_resolution {
        OV3660Resolution::HighRes => framesize_t_FRAMESIZE_QXGA,
        OV3660Resolution::MedRes => framesize_t_FRAMESIZE_SVGA,
        OV3660Resolution::LowRes => framesize_t_FRAMESIZE_QVGA,
    };

    camera_config.jpeg_quality = match user_config.format {
        OV3660Format::JPEG { quality } => quality.clamp(4, 63) as i32, // Clamp to safe range
        _ => 0,
    };

    camera_config.fb_count = if user_config.double_buffered { 2 } else { 1 };
    camera_config.fb_location = 0; 
    camera_config.grab_mode = 0;   
    camera_config.ledc_timer = ledc_timer_t_LEDC_TIMER_0;
    camera_config.ledc_channel = ledc_channel_t_LEDC_CHANNEL_0;

    // --- UNSAFE BLOCK FOR UNION & INIT ---
    unsafe {
        // Union assignments MUST be inside unsafe
        camera_config.__bindgen_anon_1.pin_sccb_sda = pinout::SIOD;
        camera_config.__bindgen_anon_2.pin_sccb_scl = pinout::SIOC;

        let err = esp_camera_init(&camera_config);
        if err != ESP_OK {
            anyhow::bail!("Camera init failed with error: {}", err);
        }
    }

    Ok(())
}
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
