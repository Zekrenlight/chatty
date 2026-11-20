//! Chatty 1.0.0 for Sunrise, by Zekrenlight (MIT).
//!
//! - Heads: the sender's face (with its hat layer) in front of their name on chat lines.
//!   The sender is read from the line (`<Name>`, `Name:`, `[Rank] Name »`, ...) and matched
//!   against the player list; the face is the game's own text head, so it wraps, fades and
//!   copies like the rest of the line.
//! - Timestamps: the local time in front of each message, 12 or 24 hour, the whole date on
//!   hover.
//! - Repeats: the same message again raises a count, `(x3)`, instead of adding lines.
//! - History: the chat and what you sent are kept when you leave, per server and per world,
//!   and come back when you return. Clear one server's or all of it from the settings or with
//!   `/chatty clear` and `/chatty clear all`.
//! - Symbols: a button at the end of the chat box opens pages of symbols to type.
//!
//! This crate holds what needs no game types: the settings (drawn by the game on the mod's
//! page, which Mod Menu opens from Options > Mods...), reading a line's sender, the time's
//! wording, the symbol pages and the history's file names. The game half is
//! `src/native/glue.rs` with the helpers in `src/mixins/`.

pub mod config;
pub mod history;
pub mod sender;
pub mod stamp;
pub mod symbols;

pub use config::*;

use mod_api::{
    ClientHost, ClientTickPostEvent, Command, Hooks, RegisterClientCommandsEvent, Registrar,
    RegistrationContext, SettingChangedEvent, SettingSpec,
};

#[derive(Default)]
struct Chatty;

fn label(key: &str) -> String {
    format!("chatty.setting.{key}")
}

fn toggle(registrar: &mut dyn Registrar, key: &str, default: bool) {
    let l = label(key);
    let tip = format!("{l}.tooltip");
    let _ = registrar.register_setting(SettingSpec::toggle(key, l, default).tooltip(tip));
}

fn range(registrar: &mut dyn Registrar, key: &str, min: f64, max: f64, step: f64, default: f64) {
    let l = label(key);
    let tip = format!("{l}.tooltip");
    let _ = registrar
        .register_setting(SettingSpec::range(key, l, min, max, step, default).tooltip(tip));
}

fn choice(registrar: &mut dyn Registrar, key: &str, options: &[&str], default: usize) {
    let l = label(key);
    let tip = format!("{l}.tooltip");
    let _ = registrar.register_setting(
        SettingSpec::choice(key, l, options.iter().copied(), default).tooltip(tip),
    );
}

fn color(registrar: &mut dyn Registrar, key: &str, default: u32) {
    let l = label(key);
    let tip = format!("{l}.tooltip");
    let _ = registrar.register_setting(SettingSpec::color(key, l, default).tooltip(tip));
}

/// Reads every setting into the copy the game half uses.
fn sync(host: &dyn ClientHost) {
    let d = Config::default();
    let b = |key: &str, default: bool| host.setting_bool(key).unwrap_or(default);
    let n = |key: &str, default: f64| {
        host.setting_f64(key)
            .filter(|v| v.is_finite())
            .unwrap_or(default)
    };
    let c = |key: &str| host.setting_choice(key).unwrap_or(0);
    set_config(Config {
        heads: b(KEY_HEADS, d.heads),
        heads_hat: b(KEY_HEADS_HAT, d.heads_hat),
        heads_own: b(KEY_HEADS_OWN, d.heads_own),
        heads_place: HeadPlace::from_index(c(KEY_HEADS_PLACE)),
        heads_known_only: b(KEY_HEADS_KNOWN, d.heads_known_only),
        timestamps: b(KEY_STAMPS, d.timestamps),
        clock: Clock::from_index(c(KEY_STAMP_CLOCK)),
        seconds: b(KEY_STAMP_SECONDS, d.seconds),
        stamp_color: host
            .setting_color(KEY_STAMP_COLOR)
            .unwrap_or(d.stamp_color)
            & 0xFF_FFFF,
        stack: b(KEY_STACK, d.stack),
        history: b(KEY_HISTORY, d.history),
        history_sent: b(KEY_HISTORY_SENT, d.history_sent),
        history_divider: b(KEY_HISTORY_DIVIDER, d.history_divider),
        history_max: n(KEY_HISTORY_MAX, d.history_max as f64).clamp(HISTORY_MIN, HISTORY_MAX)
            as usize,
        symbols: b(KEY_SYMBOLS, d.symbols),
    });
}

#[mod_api::sunrise_mod]
impl Hooks for Chatty {
    fn register(&self, registrar: &mut dyn Registrar, _cx: &RegistrationContext) {
        let d = Config::default();
        // Heads
        toggle(registrar, KEY_HEADS, d.heads);
        toggle(registrar, KEY_HEADS_HAT, d.heads_hat);
        toggle(registrar, KEY_HEADS_OWN, d.heads_own);
        choice(registrar, KEY_HEADS_PLACE, &HeadPlace::OPTIONS, 0);
        toggle(registrar, KEY_HEADS_KNOWN, d.heads_known_only);
        // Timestamps
        toggle(registrar, KEY_STAMPS, d.timestamps);
        choice(registrar, KEY_STAMP_CLOCK, &Clock::OPTIONS, 0);
        toggle(registrar, KEY_STAMP_SECONDS, d.seconds);
        color(registrar, KEY_STAMP_COLOR, d.stamp_color);
        // Repeats
        toggle(registrar, KEY_STACK, d.stack);
        // History
        toggle(registrar, KEY_HISTORY, d.history);
        toggle(registrar, KEY_HISTORY_SENT, d.history_sent);
        toggle(registrar, KEY_HISTORY_DIVIDER, d.history_divider);
        range(registrar, KEY_HISTORY_MAX, HISTORY_MIN, HISTORY_MAX, 10.0, d.history_max as f64);
        choice(registrar, KEY_CLEAR_SERVER, &CLEAR_OPTIONS, 0);
        choice(registrar, KEY_CLEAR_ALL, &CLEAR_OPTIONS, 0);
        // Symbols
        toggle(registrar, KEY_SYMBOLS, d.symbols);
        // The accent colour of chatty's cards (Sunrise orange by default).
        let _ = registrar.register_setting(
            mod_api::SettingSpec::color("accent", "chatty.setting.accent", 0xF9801D)
                .tooltip("chatty.setting.accent.tooltip"),
        );
    }

    fn register_client_commands(
        &self,
        _host: &mut dyn ClientHost,
        event: &mut RegisterClientCommandsEvent<'_>,
    ) {
        event.commands.push(Command::new("chatty"));
        event.commands.push(Command::new("chatty clear"));
        event.commands.push(Command::new("chatty clear all"));
    }

    fn setting_changed(&self, host: &mut dyn ClientHost, event: &mut SettingChangedEvent<'_>) {
        if event.mod_id == ID {
            sync(host);
        }
    }

    fn client_tick_post(&self, host: &mut dyn ClientHost, _event: &mut ClientTickPostEvent) {
        // The first tick reads the settings; later changes come through `setting_changed`.
        if !config_loaded() {
            sync(host);
        }
    }
}
