extern crate clap;

mod alsa_midi;
mod launchpad;

use clap::App;
use std::{ error, thread, time };
use launchpad::PadArea;

fn main() -> Result<(), Box<dyn error::Error>> {
    let _matches = App::new("Blinkenpad")
        .version("0.1.0")
        .author("Sunny Kalsi <thesunnyk@gmail.com>")
        .about("Blinkenlights and macropad on the Launchpad")
        .get_matches();

    let mut seq = alsa_midi::AlsaSeq::setup_alsaseq()?;
    seq.connect_all()?;
    let mut pad = launchpad::LaunchPadMini::new(&mut seq);

    let mut commands = Vec::new();

    for y in 0..8 {
        commands.push((launchpad::PadLocation::letter(y),
            launchpad::PadColour::new(0, 0)));
        commands.push((launchpad::PadLocation::number(y),
            launchpad::PadColour::new(0, 0)));
        for x in 0..8 {
            commands.push((launchpad::PadLocation::on_pad(x,y),
            launchpad::PadColour::new(0, 0)));
        }
    }
    let mut result = pad.process_io(commands)?.into_iter()
        .map(|r| (r, launchpad::PadColour::new(0, 0))).collect();
    let mut green = 0;
    let mut red = 0;
    loop {
        red = (red + 1) % 4;
        if red == 3 {
            green = (green + 1) % 4;
        }
        result = pad.process_io(result)?.into_iter()
            .map(|r| (r, launchpad::PadColour::new(red, green))).collect();
        thread::sleep(time::Duration::from_millis(100));
    }

}
