
use crate::launchpad;
use launchpad::{ PadColour, LaunchPadMini, PadLocation, PadArea};
use anyhow::{ Result, Error, Context };

/*
 * This keeps the current state of an area on the pad, and will pass those onto the actual IO.
 */
struct PadMirror {
    pad: [PadColour;64],
    letters: [PadColour;8],
    numbers: [PadColour;8]
}

impl PadMirror {
    fn new() -> PadMirror {
        PadMirror {
            pad: [PadColour::new(0,0); 64],
            letters: [PadColour::new(0,0); 8],
            numbers: [PadColour::new(0,0); 8],
        }
    }

    fn minimise(&self, set_values: Vec<(PadLocation, PadColour)>) -> Vec<(PadLocation, PadColour)> {
        set_values.into_iter().filter(|(loc, col)| match loc {
            PadLocation::OnPad(x,y) => self.pad[(*x+*y*8) as usize] != *col,
            PadLocation::Letters(l) => self.letters[*l as usize] != *col,
            PadLocation::Numbers(n) => self.numbers[*n as usize] != *col
        }).collect()
    }

    fn clear(&mut self) {
        for i in 0..self.pad.len() {
            self.pad[i] = PadColour::new(0,0);
        }
        for i in 0..self.letters.len() {
            self.letters[i] = PadColour::new(0,0);
        }
        for i in 0..self.numbers.len() {
            self.numbers[i] = PadColour::new(0,0);
        }
    }

    fn update(&mut self, set_values: &Vec<(PadLocation, PadColour)>) {
        for (loc, col) in set_values {
            match loc {
                PadLocation::OnPad(x,y) => self.pad[(*x+*y*8) as usize] = *col,
                PadLocation::Letters(l) => self.letters[*l as usize] = *col,
                PadLocation::Numbers(n) => self.numbers[*n as usize] = *col
            }
        }
    }
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
    fn process_input(&mut self, _tick: u32, set_values: &Vec<PadLocation>) -> Result<()> {
        self.locations = set_values.clone();
        Ok(())
    }
    fn process_output(&mut self, _tick: u32) -> Result<Vec<(PadLocation, PadColour)>> {
        Ok(self.locations.iter().map(|l| (l.clone(), PadColour::new(3,0))).collect())
    }
}

pub trait PluginArea {
    fn process_input(&mut self, tick: u32, set_values: &Vec<PadLocation>) -> Result<()>;
    fn process_output(&mut self, tick: u32) -> Result<Vec<(PadLocation, PadColour)>>;
}

struct PadPlugin<'a> {
    x: u8,
    y: u8,
    width: u8,
    height: u8,
    area: Box<dyn PluginArea + 'a>
}

impl PluginArea for PadPlugin<'_> {
    fn process_input(&mut self, tick: u32, set_values: &Vec<PadLocation>) -> Result<()> {
        self.area.process_input(tick,
            &set_values.into_iter()
            .filter_map(|loc| self.translate(&loc))
            .collect()
        ) 
    }

    fn process_output(&mut self, tick: u32) -> Result<Vec<(PadLocation, PadColour)>> {
        let colours = self.area.process_output(tick)?;
        let mut result = Vec::new();
        for (l, c) in colours {
            let l2 = match l {
                PadLocation::OnPad(x,y) => if x < self.width && y < self.height {
                    Ok(PadLocation::OnPad(x + self.x, y + self.y))
                } else {
                    Err(Error::msg("Outside valid area"))
                },
                PadLocation::Letters(_) => Err(Error::msg("Not on Pad")),
                PadLocation::Numbers(_) => Err(Error::msg("Not on Pad"))
            }?;
            result.push((l2, c));
        }
        Ok(result)
    }
}

impl PadPlugin<'_> {
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
    plugins: Vec<PadPlugin<'a>>,
    pad: &'a mut LaunchPadMini<'a>,
    mirror: PadMirror,
    ticks: u32
}

impl <'a> BlinkenPad<'a> {
    pub fn new(pad: &'a mut LaunchPadMini<'a>) -> BlinkenPad<'a> {
        BlinkenPad::<'a> {
            plugins: Vec::new(),
            pad: pad,
            mirror: PadMirror::new(),
            ticks: 0
        }
    }

    pub fn add_plugin(&mut self, x: u8, y: u8, width: u8, height: u8, area: Box<dyn PluginArea + 'a>) {
        self.plugins.push(
            PadPlugin {
                x: x, y: y, width: width, height: height, area: area
            }
        );
    }

    pub fn cleanup(&mut self) {
        self.plugins.clear();
    }

    pub fn clear_pad(&mut self) -> Result<()> {
        let mut commands = Vec::new();

        for y in 0..8 {
            commands.push((PadLocation::letter(y), PadColour::new(0, 0)));
            commands.push((PadLocation::number(y), PadColour::new(0, 0)));
            for x in 0..8 {
                commands.push((PadLocation::on_pad(x,y), PadColour::new(0, 0)));
            }
        }
        self.mirror.update(&commands);
        self.pad.process_in(commands)
    }

    pub fn process_all(&mut self) -> Result<bool> {
        self.ticks += 1;
        let out = self.pad.process_out()?;
        for plugin in &mut self.plugins {
            plugin.process_input(self.ticks, &out).context("On plugin input")?;
        }

        let mut lights = Vec::new();
        for plugin in &mut self.plugins {
            lights.append(&mut plugin.process_output(self.ticks).context("On plugin output")?);
        }
        if self.ticks % 50 == 0 {
            self.mirror.clear();
        }
        let min_lights = self.mirror.minimise(lights);

        self.mirror.update(&min_lights);
        self.pad.process_in(min_lights)?;
        Ok(out.contains(&PadLocation::number(7)))
    }
}

