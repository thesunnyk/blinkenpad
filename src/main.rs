extern crate clap;

mod alsa_midi;
mod launchpad;

use clap::App;
use std::{ error, thread, time };
use launchpad::PadArea;

struct PadLogger {
}

impl launchpad::PadHandler for PadLogger {
    fn on_pad(&self, location: &launchpad::PadLocation) {
        println!("Location: {:#?}", location);
    }
}

fn main() -> Result<(), Box<dyn error::Error>> {
    let _matches = App::new("Blinkenpad")
        .version("0.1.0")
        .author("Sunny Kalsi <thesunnyk@gmail.com>")
        .about("Blinkenlights and macropad on the Launchpad")
        .get_matches();

    let logger = PadLogger {};

    let mut seq = alsa_midi::AlsaSeq::setup_alsaseq()?;
    seq.connect_all()?;
    thread::sleep(time::Duration::from_millis(100));
    seq.drop_inputs()?;
    let mut pad = launchpad::LaunchPadMini::new(&mut seq);
    for y in 0..8 {
        for x in 0..8 {
            pad.set_light(launchpad::PadLocation::on_pad(x,y),
            launchpad::PadColour::new(0, 0));
        }
    }
    pad.process_io(&logger)?;
    loop {
        pad.process_io(&logger)?;
        println!("Polling");
        thread::sleep(time::Duration::from_millis(500));
    }

}
