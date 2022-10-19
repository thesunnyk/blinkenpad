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

    let seq = alsa_midi::connect_alsaseq()?;
    let mut input = seq.input();
    loop {
        thread::sleep(time::Duration::from_millis(1000));
        println!("Polling");
        while input.event_input_pending(true)? != 0 {
            let ev = input.event_input()?;
            println!("{:#?}", ev);
        }
    }

}
