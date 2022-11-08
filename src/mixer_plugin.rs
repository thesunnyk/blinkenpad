extern crate alsa;
extern crate dbus;

use anyhow::{ Result, Error };
use alsa::hctl;
use std::ffi::CString;
use crate::launchpad::{ PadColour, PadLocation};
use crate::blinken::PluginArea;

pub struct MixerPlugin {
}

impl MixerPlugin {
    pub fn new() -> Result<MixerPlugin> {

        for a in ::alsa::card::Iter::new().map(|x| x.unwrap()) {
            println!("Trying {:?}", a);
            let h = hctl::HCtl::new(format!("hw:{}", a.get_index()).as_str(), false).unwrap();
            h.load().unwrap();
            for b in h.elem_iter() {
                println!("b {:?}", b.get_id()?);
            }
        }
        Ok(MixerPlugin {})
    }
}

impl PluginArea for MixerPlugin {
    fn process_input(&mut self, tick: u32, set_values: &Vec<PadLocation>) -> Result<()> {

        Err(Error::msg("Not implemented"))
    }

    fn process_output(&mut self, tick: u32) -> Result<Vec<(PadLocation, PadColour)>> {
        Err(Error::msg("Not implemented"))
    }
}
