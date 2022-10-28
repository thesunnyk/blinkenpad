extern crate clap;

mod alsa_midi;
mod launchpad;

use clap::App;
use std::{ error, thread, time };
use alsa_midi::PadControl;
use launchpad::PadArea;

fn main() -> Result<(), Box<dyn error::Error>> {
    let _matches = App::new("Blinkenpad")
        .version("0.1.0")
        .author("Sunny Kalsi <thesunnyk@gmail.com>")
        .about("Blinkenlights and macropad on the Launchpad")
        .get_matches();

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
    pad.alsa_seq.process_io()?;
    let mut i = 0;
    loop {
        for y in 0..8 {
            for x in 0..8 {
                pad.set_light(launchpad::PadLocation::on_pad(x,y),
                launchpad::PadColour::new(x / 2, y / 2));
            }
        }
        i = i + 1;
        pad.alsa_seq.process_io()?;
        println!("Polling");
        thread::sleep(time::Duration::from_millis(500));
    }

}
