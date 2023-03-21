mod powershell_scripts;

use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener, TcpStream};
use std::process::exit;
use std::thread;
use std::time::Duration;
use bincode::{serialize, serialize_into};
use color_eyre::{Report, Result};
use serde::{Deserialize, Serialize};
use sysinfo::{Pid, PidExt, ProcessExt, SystemExt};
use crate::service::powershell_scripts::ScriptType;

const PORT: u16 = 25555;
const ADDRESS: Ipv4Addr = Ipv4Addr::LOCALHOST;
const TIMEOUT_DUR: Duration = Duration::from_secs(1);
const SERVICE_STARTUP_WAIT: Duration = Duration::from_secs(40);
const ELEVATED_SERVICE_ARG: &'static str = "elevated_service";

fn elevated_service() -> Result<()> {
    let listener = TcpListener::bind(SocketAddr::V4(SocketAddrV4::new(ADDRESS, PORT)))?;
    for stream in listener.incoming() {
        if let Ok(mut stream) = stream {
            let service_message: ClientToServiceMsg = bincode::deserialize_from(&stream)?;
            match service_message {
                ClientToServiceMsg::ProcessId(pid) => {
                    ScriptType::EnableDisplayDriver.run()?;
                    serialize_into(stream, &ServiceToClientMsg::DisplaySetupComplete)?;
                    let pid = Pid::from_u32(pid);
                    let system = sysinfo::System::new_all();
                    let client = system.process(pid).ok_or(Report::msg("client PID was incorrect"))?;
                    client.wait();
                    ScriptType::DisableDisplayDriver.run()?;
                }
            }
        }
    }
    exit(0);
    Ok(())
}
fn connect_to_service() -> Result<()> {
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
    let msg = ClientToServiceMsg::ProcessId(
        std::process::id()
    );
    bincode::serialize_into(&stream, &msg)?;
    let msg: ServiceToClientMsg = bincode::deserialize_from(&stream)?;
    match msg {
        ServiceToClientMsg::DisplaySetupComplete => {}
        _ => { return Err(Report::msg("didn't get display setup msg")); }
    }
    Ok(())
}
fn get_first_arg() -> String {
    std::env::args().collect::<Vec<_>>().first().unwrap().clone()
}
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
enum ClientToServiceMsg {
    ProcessId(u32)
}
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
enum ServiceToClientMsg {
    DisplaySetupComplete
}

pub fn init() -> Result<()> {
    if let Some(arg) = std::env::args().collect::<Vec<_>>().get(1) {
        if arg.as_str() == ELEVATED_SERVICE_ARG {
            return elevated_service();
        }
    }
    connect_to_service()?;
    Ok(())
}