extern crate alsa;
extern crate dbus;

use anyhow::{ Result, Error };
use alsa::mixer::{ Mixer, SelemId, Selem, SelemChannelId };
use crate::launchpad::{ PadColour, PadLocation};
use crate::blinken::PluginArea;
use std::rc::Rc;

pub struct MixerPlugin<'a> {
    mixer: &'a Mixer,
    master: Selem<'a>,
    capture: Selem<'a>
}

impl <'a> MixerPlugin<'a> {
    pub fn mixer() -> Result<Mixer> {
        let mut mixer = Mixer::new("pulse", true)?;
        // Selem::register(&mut mixer);
        Ok(mixer)
    }

    pub fn init(mixer: &'a Mixer) -> Result<MixerPlugin<'a>> {
        let master_id = SelemId::new("Master", 0);
        let capture_id = SelemId::new("Capture", 0);

        Ok(MixerPlugin {
            mixer: &mixer,
            master: mixer.find_selem(&master_id)
                .ok_or(Error::msg("Could not get master control"))?,
            capture: mixer.find_selem(&capture_id)
                .ok_or(Error::msg("Could not get capture control"))?
        })
    }

}

impl PluginArea for MixerPlugin<'_> {
    fn process_input(&mut self, tick: u32, set_values: &Vec<PadLocation>) -> Result<()> {
        for val in set_values {
            match val {
                PadLocation::OnPad(x,y) => {
                    match y {
                        0 => {
                            let (play_min, play_max) = self.master.get_playback_volume_range();
                            let play_set =  ((*x as i64 + 1)* (play_max - play_min)) / 8;

                            self.master.set_playback_volume_all(play_set)?;
                        },
                        1 => {
                            let (cap_min, cap_max) = self.capture.get_capture_volume_range();
                            let cap_set =  ((*x as i64 + 1) * (cap_max - cap_min)) / 8;

                            self.capture.set_capture_volume(SelemChannelId::FrontLeft, cap_set)?;
                        },
                        _ => panic!("unknown button")
                    }
                },
                PadLocation::Letters(_) => panic!("Cannot handle letters yet"),
                PadLocation::Numbers(_) => panic!("Cannot handle numbers"),
            }
        }

        Ok(())
    }

    fn process_output(&mut self, tick: u32) -> Result<Vec<(PadLocation, PadColour)>> {
        self.mixer.handle_events()?;

        let (cap_min, cap_max) = self.capture.get_capture_volume_range();
        let cap_cur = self.capture.get_capture_volume(SelemChannelId::FrontLeft)?;

        let (play_min, play_max) = self.master.get_playback_volume_range();
        let play_cur = self.master.get_playback_volume(SelemChannelId::FrontLeft)?;

        let mut result = Vec::new();

        let play_bar = ((play_cur * 8) / (play_max - play_min)) as u8;
        let cap_bar = ((cap_cur * 8) / (cap_max - cap_min)) as u8;

        for i in 0..8u8 {
            let play_col = if i < play_bar {
                PadColour::new(0,3)
            } else {
                PadColour::new(0,0)
            };
            result.push((PadLocation::OnPad(i, 0), play_col));

            let cap_col = if i < cap_bar {
                PadColour::new(2,1)
            } else {
                PadColour::new(0,0)
            };
            result.push((PadLocation::OnPad(i, 1), cap_col));
        }

        // TODO Add letter when it is supported
        Ok(result)
    }
}
