// Desktop Wallpaper Capture and Frosted Blur
use image::{imageops, RgbaImage};
use std::path::PathBuf;
use std::thread;

pub struct BlurredWallpaper {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

pub fn get_wallpaper_path() -> Option<PathBuf> {
    // Check TranscodedWallpaper in user's AppData (standard location on Windows 10 & 11)
    if let Some(appdata) = dirs::config_dir() {
        let transcoded = appdata
            .join("Microsoft")
            .join("Windows")
            .join("Themes")
            .join("TranscodedWallpaper");
        if transcoded.exists() {
            return Some(transcoded);
        }
    }

    if let Ok(appdata_env) = std::env::var("APPDATA") {
        let p = PathBuf::from(appdata_env)
            .join("Microsoft")
            .join("Windows")
            .join("Themes")
            .join("TranscodedWallpaper");
        if p.exists() {
            return Some(p);
        }
    }

    None
}

/// Loads the desktop wallpaper, downscales for fast high-quality blur,
/// and applies a dark frosted vignette tint for contrast against white text/icons.
pub fn load_blurred_wallpaper() -> Option<BlurredWallpaper> {
    let path = get_wallpaper_path()?;
    let data = std::fs::read(&path).ok()?;
    let img = image::load_from_memory(&data).ok()?.to_rgba8();

    // Downscale to 640x360 for high-performance Gaussian blur
    let target_w = 640u32;
    let target_h = (360u32 * img.height()) / img.width().max(1);
    let mut small = imageops::resize(&img, target_w, target_h, imageops::FilterType::Triangle);

    // Apply 2-pass fast box blur (approximate heavy Gaussian blur)
    fast_blur(&mut small, 12);
    fast_blur(&mut small, 12);

    // Apply dark frosted glass overlay tint: rgba(14, 18, 28, 0.55)
    for pixel in small.pixels_mut() {
        let r = pixel[0] as f32;
        let g = pixel[1] as f32;
        let b = pixel[2] as f32;

        let tint_r = 14.0;
        let tint_g = 18.0;
        let tint_b = 30.0;
        let alpha = 0.55;

        pixel[0] = (r * (1.0 - alpha) + tint_r * alpha) as u8;
        pixel[1] = (g * (1.0 - alpha) + tint_g * alpha) as u8;
        pixel[2] = (b * (1.0 - alpha) + tint_b * alpha) as u8;
        pixel[3] = 255;
    }

    Some(BlurredWallpaper {
        width: small.width(),
        height: small.height(),
        rgba: small.into_raw(),
    })
}

/// Asynchronous loader for the blurred wallpaper
pub fn spawn_wallpaper_loader<F>(on_loaded: F)
where
    F: FnOnce(BlurredWallpaper) + Send + 'static,
{
    thread::spawn(move || {
        if let Some(wallpaper) = load_blurred_wallpaper() {
            on_loaded(wallpaper);
        }
    });
}

fn fast_blur(img: &mut RgbaImage, radius: usize) {
    if radius == 0 {
        return;
    }
    let (w, h) = (img.width() as usize, img.height() as usize);
    let raw: &mut [u8] = img.as_mut();

    // Horizontal pass
    let mut temp = raw.to_vec();
    let r_i = radius as isize;
    let div = (radius * 2 + 1) as u32;

    for y in 0..h {
        let row_offset = y * w * 4;
        let mut sum_r: u32 = 0;
        let mut sum_g: u32 = 0;
        let mut sum_b: u32 = 0;

        for i in -r_i..=r_i {
            let px = i.clamp(0, (w - 1) as isize) as usize;
            let idx = row_offset + px * 4;
            sum_r += raw[idx] as u32;
            sum_g += raw[idx + 1] as u32;
            sum_b += raw[idx + 2] as u32;
        }

        for x in 0..w {
            let out_idx = row_offset + x * 4;
            temp[out_idx] = (sum_r / div) as u8;
            temp[out_idx + 1] = (sum_g / div) as u8;
            temp[out_idx + 2] = (sum_b / div) as u8;

            let left_px = (x as isize - r_i).clamp(0, (w - 1) as isize) as usize;
            let right_px = (x as isize + r_i + 1).clamp(0, (w - 1) as isize) as usize;

            let left_idx = row_offset + left_px * 4;
            let right_idx = row_offset + right_px * 4;

            sum_r = sum_r + raw[right_idx] as u32 - raw[left_idx] as u32;
            sum_g = sum_g + raw[right_idx + 1] as u32 - raw[left_idx + 1] as u32;
            sum_b = sum_b + raw[right_idx + 2] as u32 - raw[left_idx + 2] as u32;
        }
    }

    // Vertical pass
    for x in 0..w {
        let col_offset = x * 4;
        let mut sum_r: u32 = 0;
        let mut sum_g: u32 = 0;
        let mut sum_b: u32 = 0;

        for i in -r_i..=r_i {
            let py = i.clamp(0, (h - 1) as isize) as usize;
            let idx = py * w * 4 + col_offset;
            sum_r += temp[idx] as u32;
            sum_g += temp[idx + 1] as u32;
            sum_b += temp[idx + 2] as u32;
        }

        for y in 0..h {
            let out_idx = y * w * 4 + col_offset;
            raw[out_idx] = (sum_r / div) as u8;
            raw[out_idx + 1] = (sum_g / div) as u8;
            raw[out_idx + 2] = (sum_b / div) as u8;

            let top_py = (y as isize - r_i).clamp(0, (h - 1) as isize) as usize;
            let btm_py = (y as isize + r_i + 1).clamp(0, (h - 1) as isize) as usize;

            let top_idx = top_py * w * 4 + col_offset;
            let btm_idx = btm_py * w * 4 + col_offset;

            sum_r = sum_r + temp[btm_idx] as u32 - temp[top_idx] as u32;
            sum_g = sum_g + temp[btm_idx + 1] as u32 - temp[top_idx + 1] as u32;
            sum_b = sum_b + temp[btm_idx + 2] as u32 - temp[top_idx + 2] as u32;
        }
    }
}
