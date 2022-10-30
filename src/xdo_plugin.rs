
extern crate libxdo;

use libxdo::XDo;
use std::error::Error;
use crate::blinken::PluginArea;
use crate::launchpad::{PadLocation, PadColour};

pub struct XdoPlugin {
    xdo: XDo,
    colours: Vec<PadColour>,
    keys: Vec<String>
}

impl XdoPlugin {
    pub fn new(colours: Vec<PadColour>, keys: Vec<String>) -> Result<XdoPlugin, Box<dyn Error>> {
        Ok(XdoPlugin {
            xdo: XDo::new(None)?,
            colours: colours,
            keys: keys
        })
    }
}

impl PluginArea for XdoPlugin {
    fn process_input(&mut self, tick: u32, set_values: &Vec<PadLocation>) -> Result<(), Box<dyn Error>> {
        for value in set_values {
            match value {
                PadLocation::OnPad(x,y) => self.xdo.send_keysequence(&self.keys[*x as usize], 0)?,
                PadLocation::Letters(_) => panic!("Invalid letter pad press in plugin"),
                PadLocation::Numbers(_) => panic!("Invalid number pad press in plugin"),
            }
        }
        Ok(())
    }

    fn process_output(&mut self, tick: u32) -> Result<Vec<(PadLocation, PadColour)>, Box<dyn Error>> {
        let mut result = Vec::new();
        let mut x = 0;
        for c in &self.colours {
            result.push((PadLocation::on_pad(x, 0), *c));
            x += 1;
        }
        Ok(result)
    }
}

