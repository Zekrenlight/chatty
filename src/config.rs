//! The settings: their keys, defaults and the copy the game half reads.

use std::sync::Mutex;

pub const ID: &str = "chatty";

// Heads
pub const KEY_HEADS: &str = "heads";
pub const KEY_HEADS_HAT: &str = "heads_hat";
pub const KEY_HEADS_OWN: &str = "heads_own";
pub const KEY_HEADS_PLACE: &str = "heads_place";
pub const KEY_HEADS_KNOWN: &str = "heads_known_only";
// Timestamps
pub const KEY_STAMPS: &str = "timestamps";
pub const KEY_STAMP_CLOCK: &str = "timestamp_clock";
pub const KEY_STAMP_SECONDS: &str = "timestamp_seconds";
pub const KEY_STAMP_COLOR: &str = "timestamp_color";
// Repeats
pub const KEY_STACK: &str = "stack_repeats";
// History
pub const KEY_HISTORY: &str = "history";
pub const KEY_HISTORY_SENT: &str = "history_sent";
pub const KEY_HISTORY_DIVIDER: &str = "history_divider";
pub const KEY_HISTORY_MAX: &str = "history_max";
pub const KEY_CLEAR_SERVER: &str = "clear_server";
pub const KEY_CLEAR_ALL: &str = "clear_all";
// Symbols
pub const KEY_SYMBOLS: &str = "symbols";

/// The steps of a clear button: idle, waiting for the second click, done.
pub const CLEAR_OPTIONS: [&str; 3] = ["chatty.clear.idle", "chatty.clear.confirm", "chatty.clear.done"];
pub const CLEAR_IDLE: u32 = 0;
pub const CLEAR_ARMED: u32 = 1;
pub const CLEAR_DONE: u32 = 2;

/// Where a head goes on its line.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeadPlace {
    /// Right before the sender's name (before the `<` of `<Name>`).
    BeforeName,
    /// At the start of the line, after the timestamp.
    LineStart,
}

impl HeadPlace {
    pub const OPTIONS: [&'static str; 2] = ["chatty.heads_place.before_name", "chatty.heads_place.line_start"];
    pub fn from_index(i: u32) -> Self {
        match i {
            1 => Self::LineStart,
            _ => Self::BeforeName,
        }
    }
}

/// How the time reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Clock {
    /// `2:05 PM`.
    Twelve,
    /// `14:05`.
    TwentyFour,
}

impl Clock {
    pub const OPTIONS: [&'static str; 2] = ["chatty.clock.twelve", "chatty.clock.twenty_four"];
    pub fn from_index(i: u32) -> Self {
        match i {
            1 => Self::TwentyFour,
            _ => Self::Twelve,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Config {
    pub heads: bool,
    pub heads_hat: bool,
    pub heads_own: bool,
    pub heads_place: HeadPlace,
    pub heads_known_only: bool,
    pub timestamps: bool,
    pub clock: Clock,
    pub seconds: bool,
    /// `0xRRGGBB`.
    pub stamp_color: u32,
    pub stack: bool,
    pub history: bool,
    pub history_sent: bool,
    pub history_divider: bool,
    /// Messages kept per server or world.
    pub history_max: usize,
    pub symbols: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            heads: true,
            heads_hat: true,
            heads_own: true,
            heads_place: HeadPlace::BeforeName,
            heads_known_only: true,
            timestamps: true,
            clock: Clock::Twelve,
            seconds: false,
            stamp_color: 0x9D_9D97,
            stack: true,
            history: true,
            history_sent: true,
            history_divider: true,
            history_max: 100,
            symbols: true,
        }
    }
}

/// The most messages the game's chat box holds; history beyond it couldn't show.
pub const HISTORY_MAX: f64 = 100.0;
pub const HISTORY_MIN: f64 = 10.0;

static CONFIG: Mutex<Option<Config>> = Mutex::new(None);

/// The settings as last read (the defaults until the first client tick).
pub fn config() -> Config {
    CONFIG.lock().map(|c| c.unwrap_or_default()).unwrap_or_default()
}

pub fn set_config(config: Config) {
    if let Ok(mut c) = CONFIG.lock() {
        *c = Some(config);
    }
}

pub fn config_loaded() -> bool {
    CONFIG.lock().map(|c| c.is_some()).unwrap_or(false)
}
