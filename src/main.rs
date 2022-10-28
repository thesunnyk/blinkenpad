extern crate clap;

mod alsa_midi;
mod launchpad;

use clap::App;
use std::{ error, thread, time };
use launchpad::PadArea;

struct PadLooper {
    pub lights: Vec<launchpad::PadLocation>
}

impl launchpad::PadHandler for PadLooper {
    fn on_pad(&mut self, location: &launchpad::PadLocation) {
        self.lights.push(location.clone());
    }
}

fn main() -> Result<(), Box<dyn error::Error>> {
    let _matches = App::new("Blinkenpad")
        .version("0.1.0")
        .author("Sunny Kalsi <thesunnyk@gmail.com>")
        .about("Blinkenlights and macropad on the Launchpad")
        .get_matches();

    let mut looper = PadLooper { lights: Vec::new() };

    let mut seq = alsa_midi::AlsaSeq::setup_alsaseq()?;
    seq.connect_all()?;
    thread::sleep(time::Duration::from_millis(100));
    seq.drop_inputs()?;
    let mut pad = launchpad::LaunchPadMini::new(&mut seq);
    for y in 0..8 {
        pad.set_light(launchpad::PadLocation::letter(y),
            launchpad::PadColour::new(0, 0));
        pad.set_light(launchpad::PadLocation::number(y),
            launchpad::PadColour::new(0, 0));
        for x in 0..8 {
            pad.set_light(launchpad::PadLocation::on_pad(x,y),
            launchpad::PadColour::new(0, 0));
        }
    }
    pad.process_io(&mut looper)?;
    let mut green = 0;
    let mut red = 0;
    loop {
        red = (red + 1) % 4;
        if red == 3 {
            green = (green + 1) % 4;
        }
        for i in &looper.lights {
            pad.set_light(i.clone(), launchpad::PadColour::new(red, green));
        }
        looper.lights.clear();
        pad.process_io(&mut looper)?;
        thread::sleep(time::Duration::from_millis(100));
    }

}
