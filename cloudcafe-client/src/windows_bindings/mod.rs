use windows::Win32::Foundation::{HWND, POINT, RECT};
use windows::Win32::UI::WindowsAndMessaging::{GetCursorPos, GetWindowRect, IsWindow, MoveWindow, SetCursorPos};

pub type Hwnd = HWND;
pub type Rect = RECT;
pub type Point = POINT;

pub fn get_window_rect(hwnd: Hwnd) -> Rect {
    let mut rect = Rect::default();
    unsafe {
        GetWindowRect(hwnd, &mut rect);
    }
    rect
}
pub fn move_window(hwnd: Hwnd, x: i32, y: i32, width: i32, height: i32, repaint: bool) {
    unsafe {
        MoveWindow(hwnd, x, y, width, height, repaint);
    }
}
pub fn get_cursor_pos() -> Point {
    let mut point = Point::default();
    unsafe {
        GetCursorPos(&mut point);
    }
    point
}
pub fn set_cursor_pos(x: i32, y: i32) {
    unsafe {
        SetCursorPos(x, y);
    }
}
pub fn is_window(hwnd: Hwnd) -> bool {
    unsafe {
        IsWindow(hwnd)
    }.as_bool()
}