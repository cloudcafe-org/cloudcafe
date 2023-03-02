use std::thread;
use std::time::Duration;
use stereokit::Settings;

fn main() {
    let sk = Settings::default().disable_unfocused_sleep(true).init().expect("Couldn't init stereokit");
    sk.run(|sk| {

    }, |_| {});
}
