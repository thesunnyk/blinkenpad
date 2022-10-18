extern crate alsa;
extern crate clap;

use alsa::seq;
use clap::App;
use std::error;
use std::ffi::CString;

fn setup_alsaseq() -> Result<(seq::Seq, i32), Box<dyn error::Error>> {
    let seq = seq::Seq::open(None, Some(alsa::Direction::Capture), true)?;
    seq.set_client_name(&CString::new("Blinkenpad")?)?;

    let mut dinfo = seq::PortInfo::empty()?;
    dinfo.set_capability(seq::PortCap::WRITE | seq::PortCap::SUBS_WRITE);
    dinfo.set_type(seq::PortType::MIDI_GENERIC | seq::PortType::APPLICATION);
    dinfo.set_name(&CString::new("Input")?);
    seq.create_port(&dinfo)?;
    let input_port = dinfo.get_port();
    Ok((seq, input_port))
}

fn main() -> Result<(), Box<dyn error::Error>> {
    let matches = App::new("Blinkenpad")
        .version("0.1.0")
        .author("Sunny Kalsi <thesunnyk@gmail.com>")
        .about("Blinkenlights and macropad on the Launchpad")
        .get_matches();

    let (seq, port) = setup_alsaseq()?;
    let mut input = seq.input();
    loop {
        println!("Polling");
        while input.event_input_pending(true)? != 0 {
            let ev = input.event_input()?;
            println!("{:#?}", ev);
        }
    }

}
