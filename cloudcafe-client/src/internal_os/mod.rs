use windows::Win32::Graphics::Gdi::HMONITOR;
use crate::values::{IVec2, UVec2};
use crate::windows_bindings::Rect;
use color_eyre::{Report, Result};

pub mod internal_mouse;
pub mod internal_window;

#[derive(Copy, Clone, Debug)]
pub struct FakeMonitor {
    pub handle: HMONITOR,
    rect: Rect,
    pub pos: IVec2,
    pub size: UVec2,
}
impl FakeMonitor {
    pub fn new(handle: HMONITOR, rect: Rect) -> Result<Self> {
        let w = rect.right - rect.left;
        let h = rect.bottom - rect.top;
        if w < 0 || h < 0 {
            return Err(Report::msg("invalid monitor size, width or height is less then 0"));
        }
        let size = [w as u32, h as u32].into();
        Ok(Self {
            handle,
            rect,
            pos: [rect.left, rect.top].into(),
            size,
        })
    }
}