// Icon extractor with persistent disk caching and high-resolution auto-scaling
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use image::{imageops, RgbaImage};
use windows::core::{ComInterface, HSTRING};
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::{
    DeleteObject, GetDC, GetDIBits, ReleaseDC, BITMAPINFO, BITMAPINFOHEADER, BI_RGB,
    DIB_RGB_COLORS, HBITMAP, HDC, HGDIOBJ,
};
use windows::Win32::UI::Shell::{
    SHCreateItemFromParsingName, IShellItem, IShellItemImageFactory, SIIGBF_BIGGERSIZEOK,
    SIIGBF_ICONONLY, SIIGBF_SCALEUP,
};

#[derive(Clone, Debug)]
pub struct RawIcon {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

pub fn get_cache_dir() -> PathBuf {
    let dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Winscroller2dAppview")
        .join("icon_cache_v1");
    let _ = fs::create_dir_all(&dir);
    dir
}

pub fn hash_key(key: &str) -> String {
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

pub fn get_cached_or_extract(target: &str, size: u32) -> Option<RawIcon> {
    let cache_dir = get_cache_dir();
    let file_name = format!("{}_{}.png", hash_key(target), size);
    let cache_path = cache_dir.join(&file_name);

    if cache_path.exists() {
        if let Ok(bytes) = fs::read(&cache_path) {
            if let Ok(img) = image::load_from_memory(&bytes) {
                let rgba = img.to_rgba8();
                return Some(RawIcon {
                    width: rgba.width(),
                    height: rgba.height(),
                    rgba: rgba.into_raw(),
                });
            }
        }
    }

    if let Some(icon) = extract_icon_from_parsing_name(target, size) {
        let _ = image::save_buffer(
            &cache_path,
            &icon.rgba,
            icon.width,
            icon.height,
            image::ExtendedColorType::Rgba8,
        );
        return Some(icon);
    }

    None
}

pub fn extract_icon_from_parsing_name(parsing_name: &str, target_size: u32) -> Option<RawIcon> {
    unsafe {
        let hstr = HSTRING::from(parsing_name);
        let shell_item: Result<IShellItem, _> = SHCreateItemFromParsingName(&hstr, None);

        let shell_item = match shell_item {
            Ok(item) => item,
            Err(_) => return None,
        };

        let factory: Result<IShellItemImageFactory, _> = shell_item.cast();
        let factory = match factory {
            Ok(f) => f,
            Err(_) => return None,
        };

        // Request unplated high-res icon image from Windows Shell
        let win_size = windows::Win32::Foundation::SIZE { cx: 256, cy: 256 };
        let flags = SIIGBF_ICONONLY | SIIGBF_BIGGERSIZEOK | SIIGBF_SCALEUP;
        let hbitmap = factory.GetImage(win_size, flags);

        let hbitmap = match hbitmap {
            Ok(b) => b,
            Err(_) => return None,
        };

        let raw_icon = hbitmap_to_rgba(hbitmap, 256, 256);
        let _ = DeleteObject(HGDIOBJ(hbitmap.0));

        let icon = raw_icon?;
        let img = RgbaImage::from_raw(icon.width, icon.height, icon.rgba)?;

        // Normalize, trim transparent borders, and scale to target size
        let normalized = process_and_normalize_icon(img, target_size);

        Some(RawIcon {
            width: normalized.width(),
            height: normalized.height(),
            rgba: normalized.into_raw(),
        })
    }
}

/// Trims transparent padding from UWP/Win32 icons and scales them to fill the tile cleanly
fn process_and_normalize_icon(img: RgbaImage, target_size: u32) -> RgbaImage {
    let (w, h) = img.dimensions();
    let mut min_x = w;
    let mut min_y = h;
    let mut max_x = 0;
    let mut max_y = 0;

    for y in 0..h {
        for x in 0..w {
            let p = img.get_pixel(x, y);
            if p[3] > 15 {
                if x < min_x { min_x = x; }
                if x > max_x { max_x = x; }
                if y < min_y { min_y = y; }
                if y > max_y { max_y = y; }
            }
        }
    }

    if min_x > max_x || min_y > max_y {
        return imageops::resize(&img, target_size, target_size, imageops::FilterType::Triangle);
    }

    let crop_w = (max_x - min_x + 1).max(1);
    let crop_h = (max_y - min_y + 1).max(1);

    // If icon graphic has empty padding margins, crop to graphic bounding box
    let cropped = if crop_w < (w * 92 / 100) || crop_h < (h * 92 / 100) {
        imageops::crop_imm(&img, min_x, min_y, crop_w, crop_h).to_image()
    } else {
        img
    };

    // Scale cropped graphic to fill 90% of the target icon size
    let inner_target = ((target_size as f32) * 0.90).round() as u32;
    let aspect = cropped.width() as f32 / cropped.height().max(1) as f32;
    let (fit_w, fit_h) = if aspect > 1.0 {
        (inner_target, (inner_target as f32 / aspect).round() as u32)
    } else {
        (((inner_target as f32) * aspect).round() as u32, inner_target)
    };

    let resized = imageops::resize(
        &cropped,
        fit_w.max(1),
        fit_h.max(1),
        imageops::FilterType::Lanczos3,
    );

    // Place centered on a clean transparent canvas
    let mut canvas = RgbaImage::new(target_size, target_size);
    let offset_x = (target_size.saturating_sub(fit_w)) / 2;
    let offset_y = (target_size.saturating_sub(fit_h)) / 2;
    imageops::overlay(&mut canvas, &resized, offset_x as i64, offset_y as i64);

    canvas
}

pub unsafe fn hbitmap_to_rgba(hbitmap: HBITMAP, target_w: u32, target_h: u32) -> Option<RawIcon> {
    let hdc_screen: HDC = GetDC(HWND(0));
    if hdc_screen.0 == 0 {
        return None;
    }

    let mut bmi = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: target_w as i32,
            biHeight: -(target_h as i32), // top-down
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0 as u32,
            biSizeImage: target_w * target_h * 4,
            biXPelsPerMeter: 0,
            biYPelsPerMeter: 0,
            biClrUsed: 0,
            biClrImportant: 0,
        },
        bmiColors: [windows::Win32::Graphics::Gdi::RGBQUAD::default()],
    };

    let mut buffer: Vec<u8> = vec![0u8; (target_w * target_h * 4) as usize];

    let lines = GetDIBits(
        hdc_screen,
        hbitmap,
        0,
        target_h,
        Some(buffer.as_mut_ptr() as *mut _),
        &mut bmi,
        DIB_RGB_COLORS,
    );

    let _ = ReleaseDC(HWND(0), hdc_screen);

    if lines == 0 {
        return None;
    }

    // Windows returns BGRA, convert to RGBA
    let mut has_non_zero_alpha = false;
    for chunk in buffer.chunks_exact_mut(4) {
        let b = chunk[0];
        let r = chunk[2];
        let a = chunk[3];
        chunk[0] = r;
        chunk[2] = b;
        if a > 0 {
            has_non_zero_alpha = true;
        }
    }

    // Some legacy Win32 icons are returned with alpha = 0 (RGB only).
    // If all alphas are 0, treat pure black (0,0,0) as transparent background
    // and only set alpha = 255 for actual graphic content.
    if !has_non_zero_alpha {
        for chunk in buffer.chunks_exact_mut(4) {
            let r = chunk[0];
            let g = chunk[1];
            let b = chunk[2];
            if r == 0 && g == 0 && b == 0 {
                chunk[3] = 0;
            } else {
                chunk[3] = 255;
            }
        }
    }

    Some(RawIcon {
        width: target_w,
        height: target_h,
        rgba: buffer,
    })
}
