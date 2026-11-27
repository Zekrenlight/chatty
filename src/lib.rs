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
    ClientHost, ClientTickPostEvent, Command, Host, Hooks, RegisterClientCommandsEvent, Registrar,
    RegistrationContext, SettingChangedEvent, SettingKey, SettingSpec, SettingValue,
};
use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Default)]
struct Chatty;

fn label(key: &str) -> String {
    format!("chatty.setting.{key}")
}

fn remember(keys: &mut HashMap<String, SettingKey>, key: &str, k: Option<SettingKey>) {
    if let Some(k) = k {
        keys.insert(key.to_string(), k);
    }
}

fn toggle(registrar: &mut dyn Registrar, keys: &mut HashMap<String, SettingKey>, key: &str, default: bool) {
    let l = label(key);
    let tip = format!("{l}.tooltip");
    remember(keys, key, registrar.register_setting(SettingSpec::toggle(key, l, default).tooltip(tip)).ok());
}

fn range(
    registrar: &mut dyn Registrar,
    keys: &mut HashMap<String, SettingKey>,
    key: &str,
    min: f64,
    max: f64,
    step: f64,
    default: f64,
) {
    let l = label(key);
    let tip = format!("{l}.tooltip");
    remember(
        keys,
        key,
        registrar
            .register_setting(SettingSpec::range(key, l, min, max, step, default).tooltip(tip))
            .ok(),
    );
}

fn choice(
    registrar: &mut dyn Registrar,
    keys: &mut HashMap<String, SettingKey>,
    key: &str,
    options: &[&str],
    default: usize,
) {
    let l = label(key);
    let tip = format!("{l}.tooltip");
    remember(
        keys,
        key,
        registrar
            .register_setting(SettingSpec::choice(key, l, options.iter().copied(), default).tooltip(tip))
            .ok(),
    );
}

fn color(registrar: &mut dyn Registrar, keys: &mut HashMap<String, SettingKey>, key: &str, default: u32) {
    let l = label(key);
    let tip = format!("{l}.tooltip");
    remember(keys, key, registrar.register_setting(SettingSpec::color(key, l, default).tooltip(tip)).ok());
}

static SETTING_KEYS: OnceLock<HashMap<String, SettingKey>> = OnceLock::new();

/// The owner-bound key captured at registration, for the game half's writes.
pub fn setting_key(name: &str) -> Option<SettingKey> {
    SETTING_KEYS.get()?.get(name).copied()
}

fn bool_of(host: &dyn Host, name: &str, default: bool) -> bool {
    setting_key(name).and_then(|k| host.setting_bool(k).ok()).unwrap_or(default)
}
fn f64_of(host: &dyn Host, name: &str, default: f64) -> f64 {
    setting_key(name)
        .and_then(|k| host.setting_f64(k).ok())
        .filter(|v| v.is_finite())
        .unwrap_or(default)
}
fn choice_of(host: &dyn Host, name: &str, default: u32) -> u32 {
    setting_key(name).and_then(|k| host.setting_choice(k).ok()).unwrap_or(default)
}
fn color_of(host: &dyn Host, name: &str, default: u32) -> u32 {
    setting_key(name).and_then(|k| host.setting_color(k).ok()).unwrap_or(default)
}

/// Reads every setting into the copy the game half uses.
fn sync(host: &dyn Host) {
    let d = Config::default();
    set_config(Config {
        heads: bool_of(host, KEY_HEADS, d.heads),
        heads_hat: bool_of(host, KEY_HEADS_HAT, d.heads_hat),
        heads_own: bool_of(host, KEY_HEADS_OWN, d.heads_own),
        heads_place: HeadPlace::from_index(choice_of(host, KEY_HEADS_PLACE, 0)),
        heads_known_only: bool_of(host, KEY_HEADS_KNOWN, d.heads_known_only),
        timestamps: bool_of(host, KEY_STAMPS, d.timestamps),
        clock: Clock::from_index(choice_of(host, KEY_STAMP_CLOCK, 0)),
        seconds: bool_of(host, KEY_STAMP_SECONDS, d.seconds),
        stamp_color: color_of(host, KEY_STAMP_COLOR, d.stamp_color) & 0xFF_FFFF,
        stack: bool_of(host, KEY_STACK, d.stack),
        history: bool_of(host, KEY_HISTORY, d.history),
        history_sent: bool_of(host, KEY_HISTORY_SENT, d.history_sent),
        history_divider: bool_of(host, KEY_HISTORY_DIVIDER, d.history_divider),
        history_max: f64_of(host, KEY_HISTORY_MAX, d.history_max as f64).clamp(HISTORY_MIN, HISTORY_MAX)
            as usize,
        symbols: bool_of(host, KEY_SYMBOLS, d.symbols),
    });
}

#[mod_api::sunrise_mod]
impl Hooks for Chatty {
    fn register(&self, registrar: &mut dyn Registrar, _cx: &RegistrationContext) {
        let d = Config::default();
        let mut keys = HashMap::new();
        // Heads
        toggle(registrar, &mut keys, KEY_HEADS, d.heads);
        toggle(registrar, &mut keys, KEY_HEADS_HAT, d.heads_hat);
        toggle(registrar, &mut keys, KEY_HEADS_OWN, d.heads_own);
        choice(registrar, &mut keys, KEY_HEADS_PLACE, &HeadPlace::OPTIONS, 0);
        toggle(registrar, &mut keys, KEY_HEADS_KNOWN, d.heads_known_only);
        // Timestamps
        toggle(registrar, &mut keys, KEY_STAMPS, d.timestamps);
        choice(registrar, &mut keys, KEY_STAMP_CLOCK, &Clock::OPTIONS, 0);
        toggle(registrar, &mut keys, KEY_STAMP_SECONDS, d.seconds);
        color(registrar, &mut keys, KEY_STAMP_COLOR, d.stamp_color);
        // Repeats
        toggle(registrar, &mut keys, KEY_STACK, d.stack);
        // History
        toggle(registrar, &mut keys, KEY_HISTORY, d.history);
        toggle(registrar, &mut keys, KEY_HISTORY_SENT, d.history_sent);
        toggle(registrar, &mut keys, KEY_HISTORY_DIVIDER, d.history_divider);
        range(registrar, &mut keys, KEY_HISTORY_MAX, HISTORY_MIN, HISTORY_MAX, 10.0, d.history_max as f64);
        choice(registrar, &mut keys, KEY_CLEAR_SERVER, &CLEAR_OPTIONS, 0);
        choice(registrar, &mut keys, KEY_CLEAR_ALL, &CLEAR_OPTIONS, 0);
        // Symbols
        toggle(registrar, &mut keys, KEY_SYMBOLS, d.symbols);
        // The accent colour of chatty's cards (Sunrise orange by default).
        remember(
            &mut keys,
            "accent",
            registrar
                .register_setting(
                    mod_api::SettingSpec::color("accent", "chatty.setting.accent", 0xF9801D)
                        .tooltip("chatty.setting.accent.tooltip"),
                )
                .ok(),
        );
        let _ = SETTING_KEYS.set(keys);
    }


    fn register_client_commands(
        &self,
        _host: &mut dyn ClientHost,
        event: &mut RegisterClientCommandsEvent<'_>,
    ) {
        // The real work runs in the `run_client_native_command` mixin, which
        // has `&mut Game` (needed to open the screen). These registrations
        // put the command in the client dispatcher so it is not sent to the server.
        event.commands.push(Command::new("chatty [sub:text]").executes_client(|_, _| Some(1)));
    }


    fn setting_changed(&self, host: &dyn Host, event: &mut SettingChangedEvent<'_>) {
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
