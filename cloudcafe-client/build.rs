use std::env;
use std::process::Command;

fn main() {
    // Set the icon
    let mut res = winres::WindowsResource::new();
    res.set_icon("CloudCafe.ico");

    // Set the name
    let package_name = env::var("CARGO_PKG_NAME").unwrap();
    res.set("ProductName", "CloudCafe");

    // Embed any necessary resources
    // let _ = Command::new("rust-embed")
    //     .arg("path/to/resources")
    //     .arg("-R")
    //     .arg("--target")
    //     .arg("x86_64-pc-windows-msvc")
    //     .arg("-v")
    //     .status();

    // Compile the resources
    if let Err(e) = res.compile() {
        println!("Failed to compile resources: {}", e);
    }
}