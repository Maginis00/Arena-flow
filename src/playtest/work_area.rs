//! The primary monitor's work area: the screen minus the taskbar.

use bevy::prelude::*;

/// The work area in physical pixels, where the platform reports one.
/// Call it once the window exists: before that the process may not be
/// DPI aware yet and Windows reports scaled coordinates.
#[cfg(windows)]
pub(super) fn primary_work_area() -> Option<IRect> {
    use windows_sys::Win32::Foundation::RECT;
    use windows_sys::Win32::UI::WindowsAndMessaging::{SPI_GETWORKAREA, SystemParametersInfoW};

    let mut rect = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    // SAFETY: SPI_GETWORKAREA writes one RECT through the pointer, which
    // points at a live, writable RECT for the whole call.
    let ok = unsafe { SystemParametersInfoW(SPI_GETWORKAREA, 0, (&raw mut rect).cast(), 0) };
    (ok != 0).then(|| IRect::new(rect.left, rect.top, rect.right, rect.bottom))
}

/// Other platforms: no work area, the caller falls back to the whole monitor.
#[cfg(not(windows))]
pub(super) fn primary_work_area() -> Option<IRect> {
    None
}
