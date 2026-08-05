//! Screen-region OCR using only what Windows already ships: GDI grabs the
//! pixels, `Windows.Media.Ocr` reads them. No external service, no bundled
//! model, nothing added to the installer.

/// Whether this build can do OCR at all (the frontend hides the button if not).
pub fn available() -> bool {
    cfg!(target_os = "windows")
}

#[cfg(not(target_os = "windows"))]
pub fn recognize(_x: i32, _y: i32, _width: i32, _height: i32) -> Result<String, String> {
    Err("ocr_unsupported: screen OCR is only available on Windows.".to_string())
}

#[cfg(target_os = "windows")]
pub use win::recognize;

#[cfg(target_os = "windows")]
mod win {
    use std::sync::OnceLock;

    use windows::core::HSTRING;
    use windows::Globalization::Language;
    use windows::Graphics::Imaging::{BitmapPixelFormat, SoftwareBitmap};
    use windows::Media::Ocr::OcrEngine;
    use windows::Storage::Streams::DataWriter;
    use windows_sys::Win32::Graphics::Gdi::{
        BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC,
        GetDIBits, ReleaseDC, SelectObject, BITMAPINFO, BITMAPINFOHEADER,
    };
    use windows_sys::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};

    // Declared locally so the code does not depend on how a given windows-sys
    // release names or types these constants.
    const SRCCOPY: u32 = 0x00CC_0020;
    const BI_RGB: u32 = 0;
    const DIB_RGB_COLORS: u32 = 0;

    // Windows OCR rejects images smaller than 40px on a side, and reads small
    // text much better when it is scaled up first.
    const MIN_SIDE: i32 = 8;
    const TARGET_SIDE: i32 = 120;
    const MAX_SCALE: i32 = 4;

    /// A zeroed handle — works whether windows-sys types handles as pointers or
    /// as integers, which has changed between releases.
    #[inline]
    unsafe fn nul<T>() -> T {
        std::mem::zeroed()
    }

    fn ensure_apartment() {
        static ONCE: OnceLock<()> = OnceLock::new();
        ONCE.get_or_init(|| unsafe {
            // S_FALSE / RPC_E_CHANGED_MODE are both fine: we only need *an*
            // apartment to exist so WinRT activation works on this thread.
            CoInitializeEx(std::ptr::null(), COINIT_MULTITHREADED as u32);
        });
    }

    /// Copy a rectangle of the virtual screen into a top-down BGRA buffer.
    fn capture(x: i32, y: i32, width: i32, height: i32) -> Result<Vec<u8>, String> {
        unsafe {
            let screen = GetDC(nul());
            let memory = CreateCompatibleDC(screen);
            let bitmap = CreateCompatibleBitmap(screen, width, height);
            let previous = SelectObject(memory, bitmap);

            let copied = BitBlt(memory, 0, 0, width, height, screen, x, y, SRCCOPY) != 0;

            let mut pixels = vec![0u8; (width as usize) * (height as usize) * 4];
            let mut info: BITMAPINFO = std::mem::zeroed();
            info.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
            info.bmiHeader.biWidth = width;
            info.bmiHeader.biHeight = -height; // negative = top-down rows
            info.bmiHeader.biPlanes = 1;
            info.bmiHeader.biBitCount = 32;
            info.bmiHeader.biCompression = BI_RGB;

            let rows = GetDIBits(
                memory,
                bitmap,
                0,
                height as u32,
                pixels.as_mut_ptr().cast(),
                &mut info,
                DIB_RGB_COLORS,
            );

            SelectObject(memory, previous);
            DeleteObject(bitmap);
            DeleteDC(memory);
            ReleaseDC(nul(), screen);

            if !copied || rows == 0 {
                return Err("capture_error: could not read that screen region.".to_string());
            }

            // BitBlt leaves the alpha channel undefined; OCR needs it opaque.
            for pixel in pixels.chunks_exact_mut(4) {
                pixel[3] = 255;
            }
            Ok(pixels)
        }
    }

    fn upscale(pixels: &[u8], width: i32, height: i32, scale: i32) -> (Vec<u8>, i32, i32) {
        if scale <= 1 {
            return (pixels.to_vec(), width, height);
        }
        let (new_w, new_h) = (width * scale, height * scale);
        let mut out = vec![0u8; (new_w as usize) * (new_h as usize) * 4];
        for row in 0..new_h {
            let src_row = (row / scale) as usize * width as usize * 4;
            let dst_row = row as usize * new_w as usize * 4;
            for col in 0..new_w {
                let src = src_row + (col / scale) as usize * 4;
                let dst = dst_row + col as usize * 4;
                out[dst..dst + 4].copy_from_slice(&pixels[src..src + 4]);
            }
        }
        (out, new_w, new_h)
    }

    fn engine() -> Result<OcrEngine, String> {
        if let Ok(found) = OcrEngine::TryCreateFromUserProfileLanguages() {
            return Ok(found);
        }
        for tag in ["pt-BR", "en-US"] {
            if let Ok(language) = Language::CreateLanguage(&HSTRING::from(tag)) {
                if let Ok(found) = OcrEngine::TryCreateFromLanguage(&language) {
                    return Ok(found);
                }
            }
        }
        Err(
            "ocr_unavailable: Windows has no OCR language pack installed. \
             Add Portuguese or English under Settings › Time & language."
                .to_string(),
        )
    }

    fn fail(what: &'static str) -> impl Fn(windows::core::Error) -> String {
        move |error| format!("ocr_error: {what} ({error})")
    }

    fn read(pixels: &[u8], width: i32, height: i32) -> Result<String, String> {
        let writer = DataWriter::new().map_err(fail("writer"))?;
        writer.WriteBytes(pixels).map_err(fail("buffer"))?;
        let buffer = writer.DetachBuffer().map_err(fail("buffer"))?;
        let bitmap =
            SoftwareBitmap::CreateCopyFromBuffer(&buffer, BitmapPixelFormat::Bgra8, width, height)
                .map_err(fail("bitmap"))?;

        let result = engine()?
            .RecognizeAsync(&bitmap)
            .map_err(fail("recognize"))?
            .get()
            .map_err(fail("recognize"))?;

        // Joining the recognised lines keeps paragraph breaks, which reads far
        // better than the single flattened string `Text()` returns.
        let lines = result.Lines().map_err(fail("lines"))?;
        let count = lines.Size().map_err(fail("lines"))?;
        let mut text = String::new();
        for index in 0..count {
            let line = lines.GetAt(index).map_err(fail("lines"))?;
            let content = line.Text().map_err(fail("lines"))?.to_string();
            if !content.trim().is_empty() {
                if !text.is_empty() {
                    text.push('\n');
                }
                text.push_str(content.trim());
            }
        }
        Ok(text)
    }

    pub fn recognize(x: i32, y: i32, width: i32, height: i32) -> Result<String, String> {
        if width < MIN_SIDE || height < MIN_SIDE {
            return Err("region_too_small: drag a larger area.".to_string());
        }
        ensure_apartment();

        let pixels = capture(x, y, width, height)?;

        let mut scale = 1;
        while width.min(height) * scale < TARGET_SIDE && scale < MAX_SCALE {
            scale += 1;
        }
        let (pixels, width, height) = upscale(&pixels, width, height, scale);

        read(&pixels, width, height)
    }
}
