
use crate::alsa_midi;

use std::error::Error;
use alsa_midi::PadControl;

pub struct PadColour {
    red: u8,
    green: u8
}

impl PadColour {
    fn to_velocity(&self) -> u8 {
        self.red & 0x3 | ((self.green & 0x3) << 4)
    }

    pub fn new(red: u8, green: u8) -> PadColour {
        assert!(red < 4);
        assert!(green < 4);
        PadColour {
            red: red,
            green: green
        }
    }
}

#[derive(Clone, Debug)]
pub enum PadLocation {
    OnPad(u8, u8),
    Letters(u8),
    Numbers(u8)
}

impl PadLocation {

    pub fn on_pad(x: u8, y: u8) -> PadLocation {
        assert!(x < 8);
        assert!(y < 8);
        PadLocation::OnPad(x, y)
    }

    pub fn letter(l: u8) -> PadLocation {
        assert!(l < 8);
        PadLocation::Letters(l)
    }

    pub fn number(n: u8) -> PadLocation {
        assert!(n < 8);
        PadLocation::Numbers(n)
    }

    fn from_event(ev: &alsa_midi::Event) -> Option<PadLocation> {
        match ev {
            alsa_midi::Event::Note { note, velocity } => {
                let x = note % 16;
                let y = note >> 4;
                if *velocity > 0 {
                    Some(if x >= 8 {
                        PadLocation::letter(y)
                    } else {
                        PadLocation::on_pad(x, y)
                    })
                } else {
                    None
                }
            },
            alsa_midi::Event::Control { param, value } => {
                if *value > 0 {
                    Some(PadLocation::number((param - 0x68).try_into().unwrap()))
                } else {
                    None
                }
            }
        }
    }

    fn to_event(&self, colour: &PadColour) -> alsa_midi::Event {
        match self {
            PadLocation::OnPad(x, y) => alsa_midi::Event::Note {
                note: x + y * 16,
                velocity: colour.to_velocity(),
            },
            PadLocation::Letters(l) => alsa_midi::Event::Note {
                note: (l * 16) + 8,
                velocity: colour.to_velocity()
            },
            PadLocation::Numbers(n) => alsa_midi::Event::Control {
                param: (0x68 + n).try_into().unwrap(),
                value: colour.to_velocity().try_into().unwrap()
            },
        }
    }
}

pub trait PadArea {
    fn process_io(&mut self, tick: u32, set_values: Vec<(PadLocation, PadColour)>) -> Result<Vec<PadLocation>, Box<dyn Error>>;
}

pub struct LaunchPadMini<'a> {
     pub alsa_seq: &'a mut dyn PadControl,
}

impl PadArea for LaunchPadMini<'_> {

    fn process_io(&mut self, _tick: u32, set_values: Vec<(PadLocation, PadColour)>) -> Result<Vec<PadLocation>, Box<dyn Error>> {
        let events = set_values.iter().map(|(x, y)| x.to_event(y)).collect();
        Ok(self.alsa_seq.process_io(events)?.iter()
            .filter_map(|x| PadLocation::from_event(x)).collect())
    }

}

impl <'a> LaunchPadMini<'a> {
    pub fn new(seq: &'a mut alsa_midi::AlsaSeq) -> LaunchPadMini<'a> {
        LaunchPadMini {
            alsa_seq: seq,
        }
    }
}
