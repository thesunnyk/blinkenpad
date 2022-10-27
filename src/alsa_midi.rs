extern crate alsa;

use alsa::seq;
use std::error;
use std::ffi::CString;
use rand::Rng;

pub trait NoteHandler {
    fn on_note(&self);
}

pub trait PadControl {
    fn set_note(&self, velocity: u8);
    fn note_handler(&self, handler: &dyn NoteHandler);
}

pub struct AlsaSeq {
    seq: seq::Seq,
    port: seq::Addr,
    queue: i32,
}

impl AlsaSeq {
    fn create_port_info() -> Result<seq::PortInfo, Box<dyn error::Error>> {
        let mut dinfo = seq::PortInfo::empty()?;
        dinfo.set_capability(seq::PortCap::WRITE | seq::PortCap::SUBS_WRITE |
            seq::PortCap::READ | seq::PortCap::SUBS_READ);
        dinfo.set_type(seq::PortType::MIDI_GENERIC | seq::PortType::APPLICATION);
        dinfo.set_name(&CString::new("Blinkenport")?);
        Ok(dinfo)
    }

    pub fn setup_alsaseq() -> Result<AlsaSeq, Box<dyn error::Error>> {
        let seq = seq::Seq::open(None, None, true)?;
        seq.set_client_name(&CString::new("Blinkenpad")?)?;

        let port_info = AlsaSeq::create_port_info()?;
        seq.create_port(&port_info)?;
        let queue = seq.alloc_queue()?;
        Ok(AlsaSeq {
            seq: seq,
            port: port_info.addr(),
            queue: queue
        })
    }

    pub fn process_io(self: &AlsaSeq) -> Result<(), Box<dyn error::Error>> {
        let mut input = self.seq.input();
        while input.event_input_pending(true)? != 0 {
            let ev = input.event_input()?;
            println!("{:#?}", ev);
            println!("{:#?}", ev.get_source());
            println!("{:#?}", ev.get_dest());
            println!("{:#?}", ev.get_time());
            println!("{:#?}", ev.get_tick());
            println!("Queue {:#?}", ev.get_queue());

        }

        let mut rng = rand::thread_rng();
        for note in 1..200 {
            let ev_note = seq::EvNote {
                channel: 0,
                note: note,
                velocity: rng.gen(),
                off_velocity: 0,
                duration: 0
            };
            let mut ev = seq::Event::new(seq::EventType::Noteon, &ev_note);

            ev.set_subs();
            ev.set_source(self.port.port);
            ev.set_queue(self.queue);
            ev.set_tag(0);
            ev.schedule_tick(0, true, 0);
            self.seq.event_output(&mut ev)?;
        }
        self.seq.drain_output()?;

        Ok(())
    }

    pub fn drop_inputs(self: &AlsaSeq) -> Result<(), Box<dyn error::Error>> {
        let input = self.seq.input();
        while input.event_input_pending(true)? != 0 {
            input.drop_input()?;
        }
        println!("Dropped");
        Ok(())
    }

    fn connect_input(self: &AlsaSeq, port: &seq::PortInfo) -> Result<(), Box<dyn error::Error>> {
            let subs = seq::PortSubscribe::empty()?;
            subs.set_sender(port.addr());
            subs.set_dest(self.port);
            println!("Input port {}, {}", port.get_client(), port.get_port());
            self.seq.subscribe_port(&subs)?;
            Ok(())
    }

    fn connect_output(self: &AlsaSeq, port: &seq::PortInfo) -> Result<(), Box<dyn error::Error>> {
            let subs = seq::PortSubscribe::empty()?;
            subs.set_sender(self.port);
            subs.set_dest(port.addr());
            println!("Output port {}, {}", port.get_client(), port.get_port());
            self.seq.subscribe_port(&subs)?;
            Ok(())
    }

    fn connect_ports(self: &AlsaSeq, client: &seq::ClientInfo) -> Result<(), Box<dyn error::Error>> {
        for from_port in seq::PortIter::new(&self.seq, client.get_client()) {
            if from_port.get_capability().contains(seq::PortCap::NO_EXPORT) {
                println!("Skipping connection to unroutable port.");
                return Ok(());
            }
            if from_port.get_capability().contains(seq::PortCap::SUBS_READ) {
                self.connect_input(&from_port)?;
            }
            if from_port.get_capability().contains(seq::PortCap::SUBS_WRITE) {
                self.connect_output(&from_port)?;
            }
        }
        Ok(())
    }

    fn find_launchpad(self: &AlsaSeq, info: &seq::ClientInfo) -> Result<(), Box<dyn error::Error>> {
        let name = info.get_name()?;
        if name == "Launchpad Mini" {
            println!("Found Launchpad: {}", name);
            self.connect_ports(info)?;
        }
        Ok(())
    }

    pub fn connect_all(self: &AlsaSeq) -> Result<(), Box<dyn error::Error>> {
        for from_info in seq::ClientIter::new(&self.seq) {
            self.find_launchpad(&from_info)?;
        }
        Ok(())
    }
}
