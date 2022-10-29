extern crate clap;

mod alsa_midi;
mod launchpad;
mod blinken;

use clap::App;
use std::{ error, thread, time };
use blinken::BlinkenPad;

fn main() -> Result<(), Box<dyn error::Error>> {
    let _matches = App::new("Blinkenpad")
        .version("0.1.0")
        .author("Sunny Kalsi <thesunnyk@gmail.com>")
        .about("Blinkenlights and macropad on the Launchpad")
        .get_matches();

    let mut seq = alsa_midi::AlsaSeq::setup_alsaseq()?;
    seq.connect_all()?;
    let mut pad = launchpad::LaunchPadMini::new(&mut seq);
    let mut blinken = BlinkenPad::new(&mut pad);

    let mut loopback = blinken::PadLoopback::new();

    blinken.add_plugin(2, 2, 4, 4, Box::new(loopback));

    blinken.clear_pad()?;

    loop {
        blinken.process_all()?;
        thread::sleep(time::Duration::from_millis(100));
    }

}
