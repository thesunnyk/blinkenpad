extern crate alsa;

use alsa::seq;
use std::error::Error;
use std::ffi::CString;

#[derive(Debug)]
pub enum Event {
    Note {
        note: u8,
        velocity: u8
    },
    Control {
        param: u32,
        value: i32
    }
}

impl Event {
    fn to_alsa_event(&self, port: i32, queue: i32) -> seq::Event {
        let mut ev = match self {
            Event::Note { note, velocity } => {
                let ev_note = seq::EvNote {
                    channel: 0,
                    note: *note,
                    velocity: *velocity,
                    off_velocity: 0,
                    duration: 0
                };
                seq::Event::new(seq::EventType::Noteon, &ev_note)
            }
            Event::Control { param, value } => {
                let ev_ctrl = seq::EvCtrl {
                    channel: 0,
                    param: *param,
                    value: *value
                };
                seq::Event::new(seq::EventType::Controller, &ev_ctrl)
            }
        };

        ev.set_subs();
        ev.set_source(port);
        ev.set_queue(queue);
        ev.set_tag(0);
        ev.schedule_tick(0, true, 0);
        ev
    }

    fn from_alsa_event(ev: &seq::Event) -> Option<Event> {
        match ev.get_type() {
            seq::EventType::Noteon => {
                let ev_note = ev.get_data::<seq::EvNote>().unwrap();
                Some(Event::Note {
                    note: ev_note.note,
                    velocity: ev_note.velocity
                })
            },
            seq::EventType::Controller => {
                let ev_ctrl = ev.get_data::<seq::EvCtrl>().unwrap();
                Some(Event::Control {
                    param: ev_ctrl.param,
                    value: ev_ctrl.value
                })
            }
            _ => None
        }
    }
}

pub trait PadControl {
    fn process_out(&mut self) -> Result<Vec<Event>, Box<dyn Error>>;
    fn process_in(&mut self, events: Vec<Event>) -> Result<(), Box<dyn Error>>;
}

pub struct AlsaSeq {
    seq: seq::Seq,
    port: seq::Addr,
    queue: i32,
}

impl PadControl for AlsaSeq {

    fn process_in(&mut self, events: Vec<Event>) -> Result<(), Box<dyn Error>> {
        for event in events {
            self.seq.event_output(&mut event.to_alsa_event(self.port.port, self.queue))?;
        }

        self.seq.drain_output()?;
        Ok(())
    }

    fn process_out(&mut self) -> Result<Vec<Event>, Box<dyn Error>> {
        let mut input = self.seq.input();
        let mut r_vec = Vec::new();
        while input.event_input_pending(true)? != 0 {
            match Event::from_alsa_event(&input.event_input()?) {
                Some(ev) => r_vec.push(ev),
                None => {}
            }
        }
        Ok(r_vec)
    }

}

impl AlsaSeq {

    fn create_port_info() -> Result<seq::PortInfo, Box<dyn Error>> {
        let mut dinfo = seq::PortInfo::empty()?;
        dinfo.set_capability(seq::PortCap::WRITE | seq::PortCap::SUBS_WRITE |
            seq::PortCap::READ | seq::PortCap::SUBS_READ);
        dinfo.set_type(seq::PortType::MIDI_GENERIC | seq::PortType::APPLICATION);
        dinfo.set_name(&CString::new("Blinkenport")?);
        Ok(dinfo)
    }

    pub fn setup_alsaseq() -> Result<AlsaSeq, Box<dyn Error>> {
        let seq = seq::Seq::open(None, None, true)?;
        seq.set_client_name(&CString::new("Blinkenpad")?)?;

        let port_info = AlsaSeq::create_port_info()?;
        seq.create_port(&port_info)?;
        let queue = seq.alloc_queue()?;
        Ok(AlsaSeq {
            seq: seq,
            port: port_info.addr(),
            queue: queue,
        })
    }

    pub fn drop_inputs(self: &AlsaSeq) -> Result<(), Box<dyn Error>> {
        let input = self.seq.input();
        while input.event_input_pending(true)? != 0 {
            input.drop_input()?;
        }
        println!("Dropped");
        Ok(())
    }

    fn connect_input(self: &AlsaSeq, port: &seq::PortInfo) -> Result<(), Box<dyn Error>> {
            let subs = seq::PortSubscribe::empty()?;
            subs.set_sender(port.addr());
            subs.set_dest(self.port);
            println!("Input port {}, {}", port.get_client(), port.get_port());
            self.seq.subscribe_port(&subs)?;
            Ok(())
    }

    fn connect_output(self: &AlsaSeq, port: &seq::PortInfo) -> Result<(), Box<dyn Error>> {
            let subs = seq::PortSubscribe::empty()?;
            subs.set_sender(self.port);
            subs.set_dest(port.addr());
            println!("Output port {}, {}", port.get_client(), port.get_port());
            self.seq.subscribe_port(&subs)?;
            Ok(())
    }

    fn connect_ports(self: &AlsaSeq, client: &seq::ClientInfo) -> Result<(), Box<dyn Error>> {
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

    fn find_launchpad(self: &AlsaSeq, info: &seq::ClientInfo) -> Result<(), Box<dyn Error>> {
        let name = info.get_name()?;
        if name == "Launchpad Mini" {
            println!("Found Launchpad: {}", name);
            self.connect_ports(info)?;
        }
        Ok(())
    }

    pub fn connect_all(self: &AlsaSeq) -> Result<(), Box<dyn Error>> {
        for from_info in seq::ClientIter::new(&self.seq) {
            self.find_launchpad(&from_info)?;
        }
        Ok(())
    }
}
