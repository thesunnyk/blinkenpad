extern crate clap;
extern crate anyhow;

mod alsa_midi;
mod launchpad;
mod blinken;
mod xdo_plugin;
mod mpris_plugin;

use clap::App;
use std::{ thread, time };
use anyhow::Result;
use blinken::BlinkenPad;
use launchpad::PadColour;

fn main() -> Result<()> {
    let _matches = App::new("Blinkenpad")
        .version("0.1.0")
        .author("Sunny Kalsi <thesunnyk@gmail.com>")
        .about("Blinkenlights and macropad on the Launchpad")
        .get_matches();

    let mut seq = alsa_midi::AlsaSeq::setup_alsaseq()?;
    seq.connect_all()?;
    let mut pad = launchpad::LaunchPadMini::new(&mut seq);
    let mut blinken = BlinkenPad::new(&mut pad);

    let loopback = blinken::PadLoopback::new();
    let xdo = xdo_plugin::XdoPlugin::new(
        vec![PadColour::new(0,3), PadColour::new(3,0), PadColour::new(3,3)],
        vec!["super".to_string(), "control+c".to_string(), "control+v".to_string()]
        )?;
    let mpris = mpris_plugin::MprisPlugin::new()?;

    blinken.add_plugin(2, 3, 4, 4, Box::new(loopback));
    blinken.add_plugin(0, 0, 3, 1, Box::new(xdo));
    blinken.add_plugin(0, 1, 8, 2, Box::new(mpris));

    blinken.clear_pad()?;

    loop {
        if blinken.process_all()? {
            break;
        }
        thread::sleep(time::Duration::from_millis(100));
    }

    blinken.clear_pad()?;

    Ok(())
}
