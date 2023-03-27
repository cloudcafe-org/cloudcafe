mod powershell_scripts;

use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener, TcpStream};
use std::ops::ControlFlow;
use std::process::exit;
use std::thread;
use std::time::Duration;
use bincode::{serialize, serialize_into};
use color_eyre::{Report, Result};
use serde::{Deserialize, Serialize};
use sysinfo::{Pid, PidExt, ProcessExt, SystemExt};
use windows::Win32::Foundation::HWND;
use winit::dpi::{PhysicalPosition, PhysicalSize, Position, Size};
use winit::event_loop;
use winit::window::Fullscreen;
use crate::service::powershell_scripts::ScriptType;
use crate::windows_bindings::{get_console_window, Hwnd};

const PORT: u16 = 25555;
const ADDRESS: Ipv4Addr = Ipv4Addr::LOCALHOST;
const TIMEOUT_DUR: Duration = Duration::from_secs(1);
const SERVICE_STARTUP_WAIT: Duration = Duration::from_secs(40);
const ELEVATED_SERVICE_ARG: &'static str = "elevated_service";
const WINIT_ARG: &'static str = "winit";

fn elevated_service() -> Result<()> {
    let listener = TcpListener::bind(SocketAddr::V4(SocketAddrV4::new(ADDRESS, PORT)))?;
    for stream in listener.incoming() {
        if let Ok(mut stream) = stream {
            let service_message: ClientToServiceMsg = bincode::deserialize_from(&stream)?;
            match service_message {
                ClientToServiceMsg::ProcessId{ client_id, window_id } => {
                    ScriptType::EnableDisplayDriver.run()?;
                    let hwnd = get_console_window().ok_or(Report::msg("unable to get console window"))?;
                    serialize_into(stream, &ServiceToClientMsg::DisplaySetupComplete(hwnd.0))?;
                    let client_pid = Pid::from_u32(client_id);
                    let window_pid = Pid::from_u32(window_id);
                    let system = sysinfo::System::new_all();
                    let client = system.process(client_pid).ok_or(Report::msg("client PID was incorrect"))?;
                    let window = system.process(window_pid).ok_or(Report::msg("window PID was incorrect"))?;
                    client.wait();
                    window.kill();
                    //std::process::Command::new("taskkill").arg("/PID").arg(format!("{}", window_id)).arg("/F").spawn().unwrap();
                    ScriptType::DisableDisplayDriver.run()?;
                    thread::sleep(Duration::from_secs(1));
                    exit(0);
                }
            }
        }
    }
    exit(0);
    Ok(())
}
fn connect_to_service() -> Result<Hwnd> {
    let mut stream = match TcpStream::connect_timeout(&SocketAddr::V4(SocketAddrV4::new(ADDRESS, PORT)), TIMEOUT_DUR) {
        Ok(stream) => stream,
        Err(_) => {
            thread::spawn(|| {
                runas::Command::new(get_first_arg())
                    .arg(ELEVATED_SERVICE_ARG)
                    .status().unwrap()
            });
            TcpStream::connect_timeout(&SocketAddr::V4(SocketAddrV4::new(ADDRESS, PORT)), SERVICE_STARTUP_WAIT)?
        }
    };
    println!("succeeded in connect");
    let mut mouse_process = std::process::Command::new(get_first_arg()).arg(WINIT_ARG).spawn()?;
    let msg = ClientToServiceMsg::ProcessId{ client_id: std::process::id(), window_id: mouse_process.id() };
    bincode::serialize_into(&stream, &msg)?;
    let msg: ServiceToClientMsg = bincode::deserialize_from(&stream)?;
    return match msg {
        ServiceToClientMsg::DisplaySetupComplete(hwnd) => {
            Ok(HWND(hwnd))
        }
        _ => { Err(Report::msg("didn't get display setup msg")) }
    }
}
fn winit_mouse_capture() -> Result<()> {
    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new().with_fullscreen(Some(Fullscreen::Borderless(None))).with_title("Cloudcafe XR Desktop").build(&event_loop).unwrap();
    event_loop.run(move |event, _, control_flow| {
        *control_flow = event_loop::ControlFlow::Wait;

        match event {
            winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = event_loop::ControlFlow::Exit,
            _ => (),
        }
    });
    Ok(())
}
fn get_first_arg() -> String {
    std::env::args().collect::<Vec<_>>().first().unwrap().clone()
}
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
enum ClientToServiceMsg {
    ProcessId{
        client_id: u32,
        window_id: u32,
    }
}
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
enum ServiceToClientMsg {
    DisplaySetupComplete(isize)
}

pub fn init() -> Result<Option<Hwnd>> {
    if let Some(arg) = std::env::args().collect::<Vec<_>>().get(1) {
        match arg.as_str() {
            ELEVATED_SERVICE_ARG => {
                elevated_service()?;
                return Ok(None)
            }
            WINIT_ARG => {
                winit_mouse_capture()?;
                return Ok(None)
            },
            _ => (),
        }
    }
    Ok(Some(connect_to_service()?))
}