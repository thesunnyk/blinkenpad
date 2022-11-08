
extern crate mpris;

use std::time::Duration;
use crate::blinken::PluginArea;
use crate::launchpad::{PadLocation, PadColour};
use mpris::{ PlayerFinder, Player, PlaybackStatus, FindingError, DBusError, LoopStatus,
             MetadataValue, TrackID };
use anyhow::{ Context, Result, Error };

pub struct MprisPlugin {
    player: Option<Player>
}

impl MprisPlugin {
    pub fn new() -> Result<MprisPlugin> {
        Ok(MprisPlugin {
            player: None,
        })
    }

    fn get_active_player() -> Result<Option<Player>> {
        let finder = PlayerFinder::new()?;
        finder.find_active().map(|p| Some(p)).or_else(|e| match e {
                FindingError::NoPlayerFound => Ok(None),
                FindingError::DBusError(dbe) => Err(dbe).context("Can't find player")
            })
    }

    fn refresh_active(&mut self, tick: u32) -> Result<&Option<Player>> {
        match &self.player {
            Some(p) => if !p.is_running() { self.player = MprisPlugin::get_active_player()?; },
            None => {
                if tick % 20 == 0 {
                    self.player = MprisPlugin::get_active_player()?;
                }
            }
        }
        Ok(&self.player)
    }

    fn command(p: &Player, x: u8) -> Result<()> {
        match x {
            0 => p.checked_previous(),
            1 => p.checked_seek_backwards(&Duration::from_secs(5)),
            2 => p.checked_play_pause(),
            3 => p.checked_stop(),
            4 => p.checked_set_loop_status(LoopStatus::None),
            5 => p.checked_set_shuffle(false),
            6 => p.checked_seek_forwards(&Duration::from_secs(5)),
            7 => p.checked_next(),
            _ => Err(DBusError::Miscellaneous("Invalid button press".to_string()))
        }.context("While running command")?;
        Ok(())
    }

    fn current_track(p: &Player) -> Result<Option<TrackID>> {
        match p.get_metadata()?.get("mpris:trackid") {
            Some(MetadataValue::String(t)) => mpris::TrackID::new(t).map(|t| Some(t))
                .map_err(|e| Error::msg(e)),
            _ => Ok(None)
        }
    }

    fn seek(p: &Player, x: u8) -> Result<()> {

        let mut tracker = p.track_progress(100)?;
        let progress = tracker.tick().progress;
        let perc = x as f64 / 8f64;

        match MprisPlugin::current_track(p)? {
            Some(track_id) => match progress.length() {
                Some(l) => p.checked_set_position(track_id, &l.mul_f64(perc)),
                None => Ok(false)
            },
            None => Ok(false)
        }?;
        Ok(())
    }

    fn render_controls(tick: u32, status: PlaybackStatus) -> Vec<(PadLocation, PadColour)> {
        let mut result = Vec::new();
        // previous track
        result.push((PadLocation::on_pad(0, 0), PadColour::new(3,3)));
        // rewind
        result.push((PadLocation::on_pad(1, 0), PadColour::new(2,2)));
        // playpause
        let flash = if ((tick / 5) % 2) == 0 { 3u8 } else { 0u8};
        let playpausecolour = match status {
            PlaybackStatus::Playing => PadColour::new(0, flash),
            PlaybackStatus::Paused => PadColour::new(flash, flash),
            PlaybackStatus::Stopped => PadColour::new(0, 3),
        };
        result.push((PadLocation::on_pad(2, 0), playpausecolour));
        // stop
        result.push((PadLocation::on_pad(3, 0), PadColour::new(3, 0)));
        // shuffle, random
        result.push((PadLocation::on_pad(4, 0), PadColour::new(1, 0)));
        result.push((PadLocation::on_pad(5, 0), PadColour::new(0, 1)));
        // ff
        result.push((PadLocation::on_pad(6, 0), PadColour::new(2, 2)));
        // next track
        result.push((PadLocation::on_pad(7, 0), PadColour::new(3, 3)));
        result
    }

    fn render_progress(tick: u32, progress: &mpris::Progress, status: PlaybackStatus) -> Result<Vec<(PadLocation, PadColour)>> {
        let mut result = Vec::new();
        match progress.length() {
            Some(d) => {
                let pos = (progress.position().as_secs() * 8 / d.as_secs()) as u8;
                for i in 0u8..8 {
                    let col = if i < pos {
                        PadColour::new(0,1)
                    } else if i == pos {
                        PadColour::new(1,3)
                    } else {
                        PadColour::new(1,1)
                    };
                    result.push((PadLocation::on_pad(i,1), col));
                }
            },
            None => {
                let pos = ((tick / 10) % 8) as u8;
                for i in 0u8..8 {
                    let col = if i == pos && status == PlaybackStatus::Playing {
                        PadColour::new(3,0)
                    } else {
                        PadColour::new(1,1)
                    };
                    result.push((PadLocation::on_pad(i,1), col));
                }
            }
        }
        Ok(result)
    }

    fn render_with_player(tick: u32, p: &Player) -> Result<Vec<(PadLocation, PadColour)>> {
        let mut result = Vec::new();
        let status = p.get_playback_status().context("Get playback status")?;
        result.append(&mut MprisPlugin::render_controls(tick, status));
        let mut tracker = p.track_progress(100)?;

        let progress = tracker.tick().progress;
        result.append(&mut MprisPlugin::render_progress(tick, progress, status).context("Progress")?);
        Ok(result)
    }
}

impl PluginArea for MprisPlugin {
    fn process_input(&mut self, tick: u32, set_values: &Vec<PadLocation>) -> Result<()> {
        let active = self.refresh_active(tick).context("While refreshing")?;
        match active {
            Some(p) =>
            for value in set_values {
                match value {
                    PadLocation::OnPad(x,y) => match y {
                        0 => MprisPlugin::command(p, *x)?,
                        1 => MprisPlugin::seek(p, *x)?,
                        _ => Err(Error::msg("Only supports two lines"))?
                    },
                    PadLocation::Letters(_) => panic!("Invalid letter pad press in plugin"),
                    PadLocation::Numbers(_) => panic!("Invalid number pad press in plugin"),
                }
            },
            None => ()
        }
        Ok(())
    }

    fn process_output(&mut self, tick: u32) -> Result<Vec<(PadLocation, PadColour)>> {
        let active = self.refresh_active(tick).context("While refreshing on output")?;
        match active {
            Some(p) => {
                MprisPlugin::render_with_player(tick, p).or(Ok(Vec::new()))
            },
            None => {
                let mut result = Vec::new();
                for i in 0..8 {
                    result.push((PadLocation::on_pad(i,0), PadColour::new(1,0)));
                    result.push((PadLocation::on_pad(i,1), PadColour::new(1,0)));
                }
                Ok(result)
            }
        }
    }
}

