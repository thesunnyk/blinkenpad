extern crate alsa;

use alsa::seq;
use std::error;
use std::ffi::CString;

pub trait NoteHandler {
    fn on_note(&self);
}

pub trait PadControl {
    fn set_note(&self, velocity: u8);
    fn note_handler(&self, handler: dyn NoteHandler);
}

pub struct AlsaPad {

}



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

pub fn connect_alsaseq() -> Result<seq::Seq, Box<dyn error::Error>> {
    let (seq, port) = setup_alsaseq()?;
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
    Ok(seq)
}
