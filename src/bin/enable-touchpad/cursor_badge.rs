//! Mouse-layer cursor badge (application layer).
//!
//! Thin safe wrapper around [`etp_ffi::cursor_badge`]: decodes the embedded
//! 16px icon asset and hands it to the FFI overlay. Visibility is driven by
//! the watchdog's expected state (`on` = layer held = badge shown).

use etp_ffi::cursor_badge;

/// Start the overlay thread with the designed 16px badge. Call once from
/// `main`; later calls are no-ops inside the FFI layer.
///
/// # Errors
///
/// Returns the FFI error message when the overlay window cannot be created.
/// The badge is cosmetic — callers log this and keep running.
pub fn start() -> Result<(), String> {
    const PNG: &[u8] = include_bytes!("../../../assets/icon_16.png");
    let mut reader = png::Decoder::new(std::io::Cursor::new(PNG))
        .read_info()
        .map_err(|e| format!("badge icon decode failed: {e}"))?;
    let mut buf = vec![0_u8; reader.output_buffer_size().ok_or("bad badge icon")?];
    let info = reader
        .next_frame(&mut buf)
        .map_err(|e| format!("badge icon decode failed: {e}"))?;
    buf.truncate(info.buffer_size());
    cursor_badge::start(buf, info.width, info.height)
        .map_err(|e| format!("badge overlay failed: {e}"))
}

/// Show (`true`) or hide (`false`) the cursor badge. Cheap and thread-safe.
pub fn set_visible(visible: bool) {
    cursor_badge::set_visible(visible);
}
