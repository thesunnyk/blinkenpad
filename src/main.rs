extern crate alsa;
extern crate clap;

use clap::{Arg, App};

fn main() {
    let matches = App::new("Blinkenpad")
        .version("0.1.0")
        .author("Sunny Kalsi <thesunnyk@gmail.com>")
        .about("Blinkenlights and macropad on the Launchpad")
        .get_matches();

}
