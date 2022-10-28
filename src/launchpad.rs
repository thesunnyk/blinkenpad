
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

#[derive(Debug)]
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
}

pub trait PadHandler {
    fn on_pad(&self, location: &PadLocation);
}

pub trait PadArea {
    fn set_light(&mut self, location: PadLocation, colour: PadColour);
    fn process_io(&mut self, handler: &dyn PadHandler) -> Result<(), Box<dyn Error>>;
}

pub struct NoteMigrator<'a> {
    pad_handler: &'a dyn PadHandler
}

impl <'a> alsa_midi::NoteHandler for NoteMigrator<'a> {
    fn on_note(&self, note: &alsa_midi::Note) {
        let x = note.note % 16;
        let y = note.note >> 4;
        let location = if x >= 8 {
            PadLocation::letter(y)
        } else {
            PadLocation::on_pad(x, y)
        };
        self.pad_handler.on_pad(&location);
    }
}

pub struct LaunchPadMini<'a> {
    pub alsa_seq: &'a mut dyn PadControl,
}

impl PadArea for LaunchPadMini<'_> {
    fn set_light(&mut self, location: PadLocation, colour: PadColour) {
        match location {
            PadLocation::OnPad(x, y) => self.alsa_seq.set_note(alsa_midi::Note {
                note: x + y * 16,
                velocity: colour.to_velocity(),
            }),
            PadLocation::Letters(l) => self.alsa_seq.set_note(alsa_midi::Note {
                note: (l + 1) * 8,
                velocity: colour.to_velocity()
            }),
            PadLocation::Numbers(n) => println!("Tried to set a number")
        }
    }

    fn process_io(&mut self, handler: &dyn PadHandler) -> Result<(), Box<dyn Error>>{
        self.alsa_seq.process_io(&NoteMigrator {
            pad_handler: handler
        })
    }

}

impl <'a> LaunchPadMini<'a> {
    pub fn new(seq: &'a mut alsa_midi::AlsaSeq) -> LaunchPadMini<'a> {
        LaunchPadMini {
            alsa_seq: seq,
        }
    }
}
