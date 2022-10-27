extern crate clap;

mod alsa_midi;
mod launchpad;

use clap::App;
use std::{ error, thread, time };

fn main() -> Result<(), Box<dyn error::Error>> {
    let matches = App::new("Blinkenpad")
        .version("0.1.0")
        .author("Sunny Kalsi <thesunnyk@gmail.com>")
        .about("Blinkenlights and macropad on the Launchpad")
        .get_matches();

    let seq = alsa_midi::AlsaSeq::setup_alsaseq()?;
    seq.connect_all()?;
        thread::sleep(time::Duration::from_millis(1000));
    seq.drop_inputs()?;
    loop {
        println!("Polling");
        seq.process_io()?;
        thread::sleep(time::Duration::from_millis(1000));
    }

}
