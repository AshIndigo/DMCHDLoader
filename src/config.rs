use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

pub static CONFIG: LazyLock<Config> = LazyLock::new(|| {
    randomizer_utilities::load_config("dmc_loader").unwrap_or_else(|err| {
        log::error!("Failed to load config: {}", err);
        Config::default()
    })
});
#[derive(Serialize, Deserialize, Debug)]
pub struct Mods {
    pub disable_ddmk: bool,    // Stop DDMK from loading
    pub disable_crimson: bool, // Stop Crimson from loading
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub mods: Mods,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            mods: Mods {
                disable_ddmk: false,
                disable_crimson: false,
            },
        }
    }
}
