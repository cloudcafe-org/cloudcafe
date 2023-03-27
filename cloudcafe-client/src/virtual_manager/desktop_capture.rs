use std::sync::{Arc, Mutex};
use dxcapture::{Capture, Device};
use stereokit::lifecycle::StereoKitContext;
use crate::internal_os::FakeMonitor;
use color_eyre::{Report, Result};
use color_eyre::eyre::Context;

#[derive(Clone)]
pub struct CaptureDesktop(pub Arc<Mutex<CaptureDesktopInternal>>);

impl CaptureDesktop {
    pub fn new(sk: &impl StereoKitContext, fake_monitor: FakeMonitor) -> Result<Self> {
        let internal = CaptureDesktopInternal::new(sk, fake_monitor)?;
        Ok(Self(Arc::new(Mutex::new(internal))))
    }
}

pub struct CaptureDesktopInternal {
    pub capture: Capture,
    device: Device,
    fake_monitor: FakeMonitor,
}
impl CaptureDesktopInternal {
    pub fn new(sk: &impl StereoKitContext, fake_monitor: FakeMonitor) -> Result<Self> {
        let device = Device::new_from_handle(fake_monitor.handle.0).map_err(|e| Report::msg(format!("{}", e))).wrap_err("monitor device")?;
        let capture = Capture::new(&device).map_err(|e| Report::msg(format!("{}", e))).wrap_err("monitor capture")?;
        Ok(Self {
            capture,
            device,
            fake_monitor,
        })
    }
}