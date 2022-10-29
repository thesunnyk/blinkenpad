
use crate::launchpad;
use launchpad::{ PadColour, LaunchPadMini, PadLocation, PadArea};
use std::error::Error;

/*
 * This keeps the current state of an area on the pad as well as dirty flags, and will pass those
 * onto the actual IO.
 * This is for the main pad area only. Letters and numbers are not counted.
 */
struct PadMirror {
    width: u8,
    height: u8,
    pad: Vec<PadColour>,
    dirty: Vec<(u8, u8)>
}

pub struct PadLoopback {
    locations: Vec<PadLocation>
}

impl PadLoopback {
    pub fn new() -> PadLoopback {
        PadLoopback {
            locations: Vec::new()
        }
    }
}

impl PluginArea for PadLoopback {
    fn process_input(&mut self, tick: u32, set_values: &Vec<PadLocation>) -> Result<(), Box<dyn Error>> {
        self.locations = set_values.clone();
        Ok(())
    }
    fn process_output(&mut self, tick: u32) -> Result<Vec<(PadLocation, PadColour)>, Box<dyn Error>> {
        Ok(self.locations.iter().map(|l| (l.clone(), PadColour::new(3,0))).collect())
    }
}

pub trait PluginArea {
    fn process_input(&mut self, tick: u32, set_values: &Vec<PadLocation>) -> Result<(), Box<dyn Error>>;
    fn process_output(&mut self, tick: u32) -> Result<Vec<(PadLocation, PadColour)>, Box<dyn Error>>;
}

struct PadPlugin {
    x: u8,
    y: u8,
    width: u8,
    height: u8,
    area: Box<dyn PluginArea>
}

impl PluginArea for PadPlugin {
    fn process_input(&mut self, tick: u32, set_values: &Vec<PadLocation>) -> Result<(), Box<dyn Error>> {
        self.area.process_input(tick,
            &set_values.into_iter()
            .filter_map(|loc| self.translate(&loc))
            .collect()
        ) 
    }

    fn process_output(&mut self, tick: u32) -> Result<Vec<(PadLocation, PadColour)>, Box<dyn Error>> {
// TODO Translate values coming out
        let colours = self.area.process_output(tick)?;
        let mut result = Vec::new();
        for (l, c) in colours {
            let l2 = match l {
                PadLocation::OnPad(x,y) => Ok(PadLocation::OnPad(x + self.x, y + self.y)),
                PadLocation::Letters(_) => Err("Not on Pad"),
                PadLocation::Numbers(_) => Err("Not on Pad")
            }?;
            result.push((l2, c));
        }
        Ok(result)
    }
}

impl PadPlugin {
    fn translate(&self, loc: &PadLocation) -> Option<PadLocation> {
        match loc {
            PadLocation::Letters(_) => None,
            PadLocation::Numbers(_) => None,
            PadLocation::OnPad(x, y) => if *x < self.x || *x >= self.x + self.width
                || *y < self.y || *y >= self.y + self.height {
                    None
            } else {
                Some(PadLocation::OnPad(*x - self.x, *y - self.y))
            }
        }
    }

}

/*
 * The entire blinkenPad with plugins etc.
 */
pub struct BlinkenPad<'a> {
    plugins: Vec<PadPlugin>,
    pad: &'a mut LaunchPadMini<'a>,
    ticks: u32
}

impl <'a> BlinkenPad<'a> {
    pub fn new(pad: &'a mut LaunchPadMini<'a>) -> BlinkenPad<'a> {
        BlinkenPad::<'a> {
            plugins: Vec::new(),
            pad: pad,
            ticks: 0
        }
    }

    pub fn add_plugin(&mut self, x: u8, y: u8, width: u8, height: u8, area: Box<dyn PluginArea>) {
        self.plugins.push(
            PadPlugin {
                x: x, y: y, width: width, height: height, area: area
            }
        );
    }

    pub fn clear_pad(&mut self) -> Result<(), Box<dyn Error>> {
        let mut commands = Vec::new();

        for y in 0..8 {
            commands.push((PadLocation::letter(y), PadColour::new(0, 0)));
            commands.push((PadLocation::number(y), PadColour::new(0, 0)));
            for x in 0..8 {
                commands.push((PadLocation::on_pad(x,y), PadColour::new(0, 0)));
            }
        }
        self.ticks += 1;
        self.pad.process_io(self.ticks, commands)?;
        Ok(())
    }

    pub fn process_all(&mut self) -> Result<(), Box<dyn Error>> {
        self.ticks += 1;
        let mut lights = Vec::new();
        for plugin in &mut self.plugins {
            lights.append(&mut plugin.process_output(self.ticks)?);
        }
        let out = self.pad.process_io(self.ticks, lights)?;
        for plugin in &mut self.plugins {
            plugin.process_input(self.ticks, &out)?;
        }
        Ok(())
        // TODO Split input and output
    }
}

