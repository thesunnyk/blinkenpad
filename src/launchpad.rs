
use crate::alsa_midi;

use alsa_midi::PadControl;
use std::error::Error;

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

    pub fn on_letter(l: u8) -> PadLocation {
        assert!(l < 8);
        PadLocation::Letters(l)
    }
}

pub trait PadHandler {
    fn on_pad(&self, location: &PadLocation);
}

pub trait PadArea {
    fn set_light(&mut self, location: PadLocation, colour: PadColour);
    fn set_handler(&self, handler: &dyn PadHandler);
}

pub struct LaunchPadMini<'a> {
    pub alsa_seq: &'a mut alsa_midi::AlsaSeq
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


    fn set_handler(&self, handler: &dyn PadHandler) {
    }

}

impl alsa_midi::NoteHandler for LaunchPadMini<'_> {
    fn on_note(&self, note: &alsa_midi::Note) {
        println!("note {:#?}", note);
    }
}

impl LaunchPadMini<'_> {
    pub fn new(seq: &mut alsa_midi::AlsaSeq) -> LaunchPadMini {
        let mini = LaunchPadMini {
            alsa_seq: seq
        };
        // seq.note_handler(&mini);
        mini
    }
}
