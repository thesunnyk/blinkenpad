extern crate alsa;
extern crate clap;

use alsa::seq;
use clap::App;
use std::{ error, thread, time };
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
    for from_info in seq::ClientIter::new(&seq) {
        for from_port in seq::PortIter::new(&seq, from_info.get_client()) {
            if from_port.get_capability().contains(seq::PortCap::SUBS_READ)
                && !from_port.get_capability().contains(seq::PortCap::NO_EXPORT) {

                let sender = seq::Addr { client: from_port.get_client(), port: from_port.get_port() };
                let subs = seq::PortSubscribe::empty()?;
                subs.set_sender(sender);
                subs.set_dest(seq::Addr { client: seq.client_id()?, port: port });
                println!("Subscribing port {}, {}", from_port.get_client(), from_port.get_port());
                seq.subscribe_port(&subs)?;
            }
        }
    }
    loop {
        thread::sleep(time::Duration::from_millis(1000));
        println!("Polling");
        while input.event_input_pending(true)? != 0 {
            let ev = input.event_input()?;
            println!("{:#?}", ev);
        }
    }

}
