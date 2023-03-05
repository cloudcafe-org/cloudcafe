use std::ptr::null;
use stereokit::texture::TextureFormat;
use tokio::runtime;
use tokio::runtime::Runtime;
use windows::core::Abi;
use windows::Graphics::Capture;
use windows::Graphics::Capture::{GraphicsCaptureAccess, GraphicsCaptureItem, GraphicsCapturePicker, GraphicsCaptureSession};
use windows::Media::Capture::{MediaCapture, MediaCaptureSettings};
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi;
use windows::Win32::Graphics::Gdi::{BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetBitmapBits, ReleaseDC, SelectObject, SRCCOPY};
use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN};

pub fn run() {
    second();
    return;
    unsafe {
        let hdc = Gdi::GetDC(HWND::default());
        let hDest = CreateCompatibleDC(hdc);

        let height = GetSystemMetrics(SM_CYVIRTUALSCREEN);
        let width = GetSystemMetrics(SM_CXVIRTUALSCREEN);

        let hbDesktop = CreateCompatibleBitmap(hdc, width, height);


        SelectObject(hDest, hbDesktop);
        BitBlt(hDest, 0, 0, width, height, hdc, 0, 0, SRCCOPY);
        ReleaseDC(HWND::default(), hdc);
        DeleteObject(hbDesktop);
        DeleteDC(hDest);
    }
    //tokio::spawn(async || {
       // let item = picker.PickSingleItemAsync().unwrap().await.unwrap();
    //});
    //let device = MediaCapture::new().unwrap().MediaCaptureSettings().unwrap().Direct3D11Device().unwrap();
    //MediaCaptureSettings;
    //Capture::Direct3D11CaptureFramePool::Create(, Default::default(), 0, Default::default());
}

pub fn second() {
    // let hwinds = win_screenshot::utils::window_list().unwrap();
    // let hwinds: Vec<_> = hwinds.into_iter().map(|a| {
    //    a.hwnd
    // }).collect();
    // let hwnds: Vec<_> = hwinds.into_iter().map(|a| {
    //    HWND(a)
    // }).collect();
    //println!("winds: {:?}", hwinds);
    let rt = runtime::Builder::new_current_thread().build().unwrap();
    rt.block_on(async move {
        if !GraphicsCaptureSession::IsSupported().unwrap() {
            println!("not supported");
            return;
        }
        //GraphicsCaptureSession::StartCapture(&()).unwrap();
        let picker = GraphicsCapturePicker::new().unwrap();
        let item = picker.PickSingleItemAsync().unwrap().await.unwrap();
    });

}