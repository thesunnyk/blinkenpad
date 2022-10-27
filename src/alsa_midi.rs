extern crate alsa;

use alsa::seq;
use std::error;
use std::ffi::CString;

pub trait NoteHandler {
    fn on_note(&self);
}

pub trait PadControl {
    fn set_note(&self, velocity: u8);
    fn note_handler(&self, handler: &dyn NoteHandler);
}

pub struct AlsaSeq {
    pub seq: seq::Seq,
    input_port: i32,
    output_port: i32,
}

impl AlsaSeq {
    fn create_input_port_info() -> Result<seq::PortInfo, Box<dyn error::Error>> {
        let mut dinfo = seq::PortInfo::empty()?;
        dinfo.set_capability(seq::PortCap::WRITE | seq::PortCap::SUBS_WRITE);
        dinfo.set_type(seq::PortType::MIDI_GENERIC | seq::PortType::APPLICATION);
        dinfo.set_name(&CString::new("Input")?);
        Ok(dinfo)
    }

    pub fn setup_alsaseq() -> Result<AlsaSeq, Box<dyn error::Error>> {
        let seq = seq::Seq::open(None, None, true)?;
        seq.set_client_name(&CString::new("Blinkenpad")?)?;

        let dinfo = AlsaSeq::create_input_port_info()?;
        seq.create_port(&dinfo)?;
        Ok(AlsaSeq {
            seq: seq,
            input_port: dinfo.get_port(),
            output_port: 0
        })
    }

    pub fn process_io(self: &AlsaSeq) -> Result<(), Box<dyn error::Error>> {
        let mut input = self.seq.input();
        while input.event_input_pending(true)? != 0 {
            let ev = input.event_input()?;
            println!("{:#?}", ev);
        }
        Ok(())
    }

    fn connect_port(self: &AlsaSeq, from_port: &seq::PortInfo) -> Result<(), Box<dyn error::Error>> {
        if from_port.get_capability().contains(seq::PortCap::NO_EXPORT) {
            println!("Attempting connection to unroutable port.");
            return Ok(());
        }
        if from_port.get_capability().contains(seq::PortCap::SUBS_READ) {
            let sender = seq::Addr { client: from_port.get_client(), port: from_port.get_port() };
            let subs = seq::PortSubscribe::empty()?;
            subs.set_sender(sender);
            subs.set_dest(seq::Addr { client: self.seq.client_id()?, port: self.input_port });
            println!("Input port {}, {}", from_port.get_client(), from_port.get_port());
            self.seq.subscribe_port(&subs)?;
        }
        Ok(())
    }

    fn connect_client(self: &AlsaSeq, client: &seq::ClientInfo) -> Result<(), Box<dyn error::Error>> {
            for from_port in seq::PortIter::new(&self.seq, client.get_client()) {
                self.connect_port(&from_port)?;
            }
            Ok(())
    }

    fn find_launchpad(self: &AlsaSeq, info: &seq::ClientInfo) -> Result<(), Box<dyn error::Error>> {
        let name = info.get_name()?;
        if name == "Launchpad Mini" {
            println!("Found Launchpad: {}", name);
            self.connect_client(&info)?;
        }
        Ok(())
    }

    pub fn connect_all(self: &AlsaSeq) -> Result<(), Box<dyn error::Error>> {
        for from_info in seq::ClientIter::new(&self.seq) {
            self.find_launchpad(&from_info)?;
        }
        self.seq.input().drop_input()?;
        Ok(())
    }
}
