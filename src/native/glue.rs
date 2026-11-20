// Chatty in the game (not in the Cargo library): heads, timestamps and repeats on the lines the
// chat receives, the history kept per server and world, the clear buttons and command, and the
// symbol picker on the chat screen.
use super::{Game, Screen};
use mod_chatty as ch;
use mod_chatty::symbols::{self, Hit, Layout, Page};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use sunrise_client::chat::{ChatSource, SYSTEM_INDICATOR};
use sunrise_client::hud::HudBatch;
use sunrise_client::text::{InlineObject, Style, Text};

/// A message kept for the history: the line as shown, where it came from, how many times in a
/// row it came, and when (Unix seconds).
struct Entry {
    text: Text,
    source: ChatSource,
    count: u32,
    time: i64,
}

/// The last line Chatty passed on: what came in, what went out, and the chat's message count
/// after it, so a repeat is only taken as one while nothing else came between.
struct Last {
    raw: Text,
    source: ChatSource,
    shown: Text,
    received: u64,
}

#[derive(Default)]
struct State {
    /// The server or world in place (`server:<address>`, `world:<save folder>`).
    session: Option<String>,
    /// The one before, for "this server" from the title screen.
    previous: Option<String>,
    entries: Vec<Entry>,
    /// What the sent history looked like when last kept: its length and last line.
    sent: (usize, Option<String>),
    dirty: bool,
    written: Option<Instant>,
    last: Option<Last>,
    /// The player list's names, lower case, and your own.
    known: HashSet<String>,
    own: String,
    known_at: Option<Instant>,
    // The symbol picker.
    open: bool,
    page: usize,
    recent: Vec<char>,
    layout: Option<Layout>,
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
}

fn with<R>(f: impl FnOnce(&mut State) -> R) -> R {
    STATE.with(|s| f(&mut s.borrow_mut()))
}

fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64)
}

// ---- the server or world in place ----------------------------------------------------------

/// The server or world the window is logged in to; None on the title screen and while joining.
fn session_key(game: &Game<'_>) -> Option<String> {
    let remote = game.remote.as_ref().filter(|r| r.logged_in)?;
    if let Some(world) = game.integrated.as_ref() {
        return Some(format!("world:{}", world.save_folder));
    }
    let address = remote.session.address.as_str();
    if address == "local" {
        return Some("world:local".to_owned());
    }
    Some(format!(
        "server:{}",
        sunrise_client::net::shown_address(address).to_ascii_lowercase()
    ))
}

fn history_dir(game: &Game<'_>) -> PathBuf {
    game.native_data_dir(ch::ID).join("history")
}

/// A server or world was joined or left: the old one's history is written, the new one's read
/// back into the chat.
fn ensure_session(game: &mut Game<'_>) {
    let key = session_key(game);
    if with(|s| s.session == key) {
        return;
    }
    flush(game);
    with(|s| {
        if let Some(old) = s.session.take() {
            s.previous = Some(old);
        }
        s.session = key.clone();
        s.entries.clear();
        s.last = None;
        s.dirty = false;
        s.written = None;
    });
    let Some(key) = key else {
        return;
    };
    let cfg = ch::config();
    let loaded = if cfg.history && game.on(ch::ID) {
        load(&history_dir(game).join(ch::history::file_name(&key)), cfg.history_max)
    } else {
        None
    };
    let (entries, sent) = loaded.unwrap_or_default();
    restore(game, entries, sent, cfg);
}

/// Puts the kept messages back in the chat, faded as old lines are, and the kept sent
/// messages back in the chat box's history.
fn restore(game: &mut Game<'_>, entries: Vec<Entry>, sent: Vec<String>, cfg: ch::Config) {
    let focused = game.screen == Screen::Chat;
    for e in &entries {
        if !game.ui.chat.shows(e.source) {
            continue;
        }
        let indicator = match e.source {
            ChatSource::Player => None,
            ChatSource::Server | ChatSource::Client => Some(SYSTEM_INDICATOR),
        };
        // With repeats stacked, the chat counts them again as they go in.
        let times = if cfg.stack { e.count.clamp(1, 64) } else { 1 };
        for _ in 0..times {
            game.ui
                .chat
                .add(&game.gui.atlas, e.text.clone(), indicator, e.source, 0, focused);
        }
    }
    if !entries.is_empty() && cfg.history_divider {
        let line = Text::translate("chatty.history.divider", &[]).with_color(0x7A_8294);
        game.ui
            .chat
            .add(&game.gui.atlas, line, Some(SYSTEM_INDICATOR), ChatSource::Client, 0, focused);
    }
    // Sent messages are the server's own; commands go everywhere.
    if cfg.history && cfg.history_sent {
        if sent.is_empty() {
            game.ui.chat.recent.retain(|m| m.starts_with('/'));
        } else {
            game.ui.chat.recent = sent;
        }
    }
    let snapshot = sent_snapshot(game);
    with(|s| {
        s.entries = entries;
        s.sent = snapshot;
    });
}

fn sent_snapshot(game: &Game<'_>) -> (usize, Option<String>) {
    let recent = &game.ui.chat.recent;
    (recent.len(), recent.last().cloned())
}

fn source_name(source: ChatSource) -> &'static str {
    match source {
        ChatSource::Player => "player",
        ChatSource::Server => "server",
        ChatSource::Client => "client",
    }
}

fn source_of(name: &str) -> ChatSource {
    match name {
        "player" => ChatSource::Player,
        "client" => ChatSource::Client,
        _ => ChatSource::Server,
    }
}

/// A history file: its messages (the newest `max`) and sent messages.
fn load(path: &std::path::Path, max: usize) -> Option<(Vec<Entry>, Vec<String>)> {
    let text = std::fs::read_to_string(path).ok()?;
    let v: Value = serde_json::from_str(&text).ok()?;
    let mut entries: Vec<Entry> = v
        .get("messages")?
        .as_array()?
        .iter()
        .filter_map(|m| {
            let component = m.get("text")?;
            Some(Entry {
                text: sunrise_client::scoreboard::comp_text(component),
                source: source_of(m.get("source").and_then(Value::as_str).unwrap_or("")),
                count: m
                    .get("count")
                    .and_then(Value::as_u64)
                    .map_or(1, |n| n.clamp(1, 9999) as u32),
                time: m.get("time").and_then(Value::as_i64).unwrap_or(0),
            })
        })
        .collect();
    let extra = entries.len().saturating_sub(max);
    entries.drain(..extra);
    let sent = v
        .get("sent")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .rev()
                .take(100)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect()
        })
        .unwrap_or_default();
    Some((entries, sent))
}

/// Writes the history in place when it changed.
pub(super) fn flush(game: &Game<'_>) {
    let cfg = ch::config();
    let job = with(|s| {
        if !s.dirty {
            return None;
        }
        s.dirty = false;
        s.written = Some(Instant::now());
        let key = s.session.clone()?;
        let messages: Vec<Value> = s
            .entries
            .iter()
            .map(|e| {
                json!({
                    "time": e.time,
                    "source": source_name(e.source),
                    "count": e.count,
                    "text": e.text.component(),
                })
            })
            .collect();
        Some((key, messages))
    });
    let Some((key, messages)) = job else {
        return;
    };
    if !cfg.history {
        return;
    }
    let sent: Vec<String> = if cfg.history_sent {
        game.ui.chat.recent.clone()
    } else {
        Vec::new()
    };
    let file = json!({ "version": 1, "key": key, "messages": messages, "sent": sent });
    let dir = history_dir(game);
    let path = dir.join(ch::history::file_name(&key));
    let temp = dir.join(format!(".{}.tmp", ch::history::file_name(&key)));
    let written = std::fs::create_dir_all(&dir)
        .and_then(|()| std::fs::write(&temp, file.to_string()))
        .and_then(|()| std::fs::rename(&temp, &path));
    if let Err(e) = written {
        log::warn!("Chatty couldn't keep the chat history in {}: {e}", path.display());
    }
}

/// The level is going (leaving, or changing dimension): the history is written now.
pub(super) fn level_unload(game: &Game<'_>) {
    flush(game);
}

/// Once a client tick: joining and leaving, the player list's names, the sent history, and
/// the history written at most every few seconds.
pub(super) fn tick(game: &mut Game<'_>) {
    if !game.on(ch::ID) {
        return;
    }
    ensure_session(game);
    refresh_known(game);
    let cfg = ch::config();
    if cfg.history && cfg.history_sent {
        let recent = &game.ui.chat.recent;
        let (len, last) = (recent.len(), recent.last());
        let changed = with(|s| s.sent.0 != len || s.sent.1.as_ref() != last);
        if changed {
            let snapshot = sent_snapshot(game);
            with(|s| {
                if s.session.is_some() {
                    s.dirty = true;
                }
                s.sent = snapshot;
            });
        }
    }
    let due = with(|s| {
        s.dirty && s.written.is_none_or(|t| t.elapsed() > Duration::from_secs(3))
    });
    if due {
        flush(game);
    }
}

// ---- lines coming in -------------------------------------------------------------------------

/// The player list's names (once a second at most) and your own.
fn refresh_known(game: &Game<'_>) {
    let stale = with(|s| s.known_at.is_none_or(|t| t.elapsed() > Duration::from_secs(1)));
    if !stale {
        return;
    }
    let mut known = HashSet::new();
    if let Some(remote) = game.remote.as_ref() {
        for p in remote.listed_tab_players() {
            known.insert(p.name.to_ascii_lowercase());
        }
    }
    // Your own name is read once: the game's launch options don't change while it runs.
    let own = with(|s| s.own.clone());
    let own = if own.is_empty() { game.options.player_name() } else { own };
    known.insert(own.to_ascii_lowercase());
    with(|s| {
        s.known = known;
        s.own = own;
        s.known_at = Some(Instant::now());
    });
}

/// A line is about to join the chat (`content` as the game would show it, from `source`,
/// `signed` for a signed player message). Returns the line to show instead, or None to leave
/// the game alone (Chatty off, or nothing of it on).
pub(super) fn incoming(
    game: &mut Game<'_>,
    content: &Text,
    source: ChatSource,
    signed: bool,
) -> Option<Text> {
    if !game.on(ch::ID) {
        return None;
    }
    ensure_session(game);
    let cfg = ch::config();
    let keep = cfg.history && with(|s| s.session.is_some());
    if !(cfg.heads || cfg.timestamps || cfg.stack || keep) {
        return None;
    }
    let received = game.ui.chat.received;
    // The same line again with nothing between: the same text goes out, so the chat raises
    // the count on the line it shows (signed messages never stack: a server may delete one).
    if cfg.stack && !signed {
        let again = with(|s| match &s.last {
            Some(l) if l.source == source && l.received == received && l.raw == *content => {
                Some(l.shown.clone())
            }
            Some(_) | None => None,
        });
        if let Some(text) = again {
            with(|s| {
                if keep && let Some(e) = s.entries.last_mut() {
                    e.count = e.count.saturating_add(1);
                    e.time = unix_now();
                    s.dirty = true;
                }
            });
            return Some(text);
        }
    }
    refresh_known(game);
    let text = compose(game, content, cfg);
    with(|s| {
        s.last = Some(Last {
            raw: content.clone(),
            source,
            shown: text.clone(),
            received: u64::MAX,
        });
        if keep {
            s.entries.push(Entry {
                text: text.clone(),
                source,
                count: 1,
                time: unix_now(),
            });
            let extra = s.entries.len().saturating_sub(cfg.history_max);
            s.entries.drain(..extra);
            s.dirty = true;
        }
    });
    Some(text)
}

/// The line `incoming` gave went into the chat: its message count is noted.
pub(super) fn shown(game: &Game<'_>) {
    let received = game.ui.chat.received;
    with(|s| {
        if let Some(l) = s.last.as_mut() {
            l.received = received;
        }
    });
}

/// The line with its timestamp and the sender's head.
fn compose(game: &Game<'_>, content: &Text, cfg: ch::Config) -> Text {
    let head = if cfg.heads { sender(game, content, cfg) } else { None };
    if head.is_none() && !cfg.timestamps {
        return content.clone();
    }
    let mut out = Text::empty();
    if cfg.timestamps {
        out = out.append(stamp(cfg));
    }
    match head {
        Some((base, at, name)) => {
            let len = base.chars().count();
            out = out
                .append(base.slice(0, at))
                .append(head_of(&name, cfg.heads_hat))
                .append(Text::literal(" "))
                .append(base.slice(at, len));
        }
        None => out = out.append(content.clone()),
    }
    out
}

/// The sender read from the line: the line with its `§` codes applied as the chat would, the
/// character to put the head before, and the name.
fn sender(game: &Game<'_>, content: &Text, cfg: ch::Config) -> Option<(Text, usize, String)> {
    let base = if game.ui.chat.options.colors {
        content.resolve_legacy()
    } else {
        content.strip_legacy()
    };
    let chars: Vec<char> = base.chars().map(|(c, _)| c).collect();
    let (found, own) = with(|s| {
        let known = |w: &str| s.known.contains(&w.to_ascii_lowercase());
        (
            ch::sender::find(&chars, &known, cfg.heads_known_only),
            s.own.clone(),
        )
    });
    let found = found?;
    if !cfg.heads_own && found.name.eq_ignore_ascii_case(&own) {
        return None;
    }
    let at = match cfg.heads_place {
        ch::HeadPlace::BeforeName => found.start,
        ch::HeadPlace::LineStart => found.line_start,
    };
    Some((base, at, found.name))
}

/// The game's own text head (`minecraft:object`, a player sprite) for `name`: the skin the
/// player list has for them, else the one their name has.
fn head_of(name: &str, hat: bool) -> Text {
    Text::styled(
        "\u{FFFC}",
        Style {
            inline_object: Some(InlineObject::Player {
                profile: Arc::new(Value::String(name.to_owned())),
                hat,
                fallback: String::new(),
            }),
            ..Style::default()
        },
    )
}

/// `[2:05 PM] `, the whole date on hover.
fn stamp(cfg: ch::Config) -> Text {
    use chrono::Timelike;
    let now = chrono::Local::now();
    let (h, m, s) = (now.hour(), now.minute(), now.second());
    let short = ch::stamp::short(cfg.clock, cfg.seconds, h, m, s);
    let date = now.format("%A %-d %B %Y").to_string();
    let long = ch::stamp::long(cfg.clock, &date, h, m, s);
    Text::styled(
        short,
        Style {
            color: Some(cfg.stamp_color),
            hover: Some(Arc::new(Text::literal(long))),
            ..Style::default()
        },
    )
}

/// PvP & HUD's chat options as the chat lays its lines out: with Chatty's repeats on the chat
/// stacks them, and with Chatty's timestamps on the chat doesn't add a second time.
pub(super) fn qol(
    game: &Game<'_>,
    mut options: sunrise_client::pvp::ChatOptions,
) -> sunrise_client::pvp::ChatOptions {
    if !game.on(ch::ID) {
        return options;
    }
    let cfg = ch::config();
    if cfg.stack {
        options.compact = true;
    }
    if cfg.timestamps {
        options.timestamps = false;
    }
    options
}

// ---- clearing --------------------------------------------------------------------------------

/// Empties the chat on screen and the sent chat lines (commands stay, as the game keeps them).
fn clear_screen(game: &mut Game<'_>) {
    game.ui.chat.clear();
    game.ui.chat.recent.retain(|m| m.starts_with('/'));
    let snapshot = sent_snapshot(game);
    with(|s| {
        s.entries.clear();
        s.last = None;
        s.dirty = false;
        s.sent = snapshot;
    });
}

/// Forgets the history of the server or world in place (or the last one, from the title
/// screen). False when there was none to name.
fn clear_this(game: &mut Game<'_>) -> bool {
    let (key, current) = with(|s| match &s.session {
        Some(k) => (Some(k.clone()), true),
        None => (s.previous.clone(), false),
    });
    let Some(key) = key else {
        return false;
    };
    let _ = std::fs::remove_file(history_dir(game).join(ch::history::file_name(&key)));
    if current {
        clear_screen(game);
    }
    true
}

/// Forgets every server's and world's history.
fn clear_all(game: &mut Game<'_>) {
    let dir = history_dir(game);
    if let Ok(files) = std::fs::read_dir(&dir) {
        for file in files.flatten() {
            let name = file.file_name();
            if ch::history::is_history_file(&name.to_string_lossy()) {
                let _ = std::fs::remove_file(file.path());
            }
        }
    }
    if with(|s| s.session.is_some()) {
        clear_screen(game);
    }
}

/// A line from Chatty itself, not kept in the history.
fn say(game: &mut Game<'_>, key: &str) {
    let focused = game.screen == Screen::Chat;
    let ticks = game.visual_ticks();
    let line = Text::translate(key, &[]).with_color(0xB8_C4DC);
    game.ui
        .chat
        .add(&game.gui.atlas, line, Some(SYSTEM_INDICATOR), ChatSource::Client, ticks, focused);
}

/// A setting changed on Chatty's page: the clear buttons. The first click asks, the second
/// clears; the kept value goes back to the start each time, so the page opens fresh.
pub(super) fn setting_changed(game: &mut Game<'_>, mod_id: &str, key: &str) {
    if mod_id != ch::ID || !(key == ch::KEY_CLEAR_SERVER || key == ch::KEY_CLEAR_ALL) {
        return;
    }
    let step = game
        .setting(ch::ID, key)
        .and_then(|v| v.trim().parse::<u32>().ok())
        .unwrap_or(ch::CLEAR_IDLE);
    if step == ch::CLEAR_ARMED {
        game.set_setting(ch::ID, key, "0");
    } else if step == ch::CLEAR_DONE {
        if key == ch::KEY_CLEAR_ALL {
            clear_all(game);
        } else {
            clear_this(game);
        }
        game.set_setting(ch::ID, key, "0");
    }
}

/// `/chatty`, `/chatty clear` and `/chatty clear all`.
pub(super) fn command(game: &mut Game<'_>, command: &str, ran: &mut bool) {
    let mut words = command.split_whitespace();
    if words.next() != Some(ch::ID) || !game.on(ch::ID) {
        return;
    }
    *ran = true;
    let rest: Vec<&str> = words.collect();
    match rest.as_slice() {
        ["clear"] => {
            let key = if clear_this(game) {
                "chatty.command.cleared"
            } else {
                "chatty.command.nothing"
            };
            say(game, key);
        }
        ["clear", "all"] => {
            clear_all(game);
            say(game, "chatty.command.cleared_all");
        }
        _ => say(game, "chatty.command.help"),
    }
}

// ---- the symbol picker -----------------------------------------------------------------------

fn picker_on(game: &Game<'_>) -> bool {
    game.on(ch::ID)
        && ch::config().symbols
        && game.screen == Screen::Chat
        && game.ui.chat_screen.is_some()
}

/// Whether the font has its own glyph for `c` (not the missing-glyph box).
fn drawable(font: &sunrise_client::font::Font, c: char) -> bool {
    let g = font.glyph(c);
    let missing = font.glyph('\u{10FFFD}');
    match (g.rect, missing.rect) {
        (None, _) => false,
        (Some(_), None) => true,
        (Some(a), Some(b)) => !(a.u0 == b.u0 && a.v0 == b.v0 && g.advance == missing.advance),
    }
}

/// The symbols page `page` shows.
fn symbols_of(game: &Game<'_>, page: Page, recent: &[char]) -> Vec<char> {
    let font = &game.gui.atlas.font;
    match page {
        Page::Recent => recent.to_vec(),
        Page::Faces
        | Page::Weather
        | Page::Arrows
        | Page::Shapes
        | Page::Gear
        | Page::Marks
        | Page::Letters
        | Page::Emoji => page
            .symbols()
            .chars()
            .filter(|&c| drawable(font, c))
            .take(symbols::COLS * symbols::ROWS)
            .collect(),
    }
}

fn glyph_in(game: &Game<'_>, batch: &mut HudBatch, c: char, r: symbols::Rect, color: u32) {
    let atlas = &game.gui.atlas;
    let advance = atlas.font.glyph(c).advance;
    let x = r.x + ((r.w - advance) / 2.0).floor() + 1.0;
    batch.text(atlas, &c.to_string(), x, r.y + 2.0, color, false);
}

fn outline(game: &Game<'_>, batch: &mut HudBatch, r: symbols::Rect, color: u32) {
    let atlas = &game.gui.atlas;
    batch.fill(atlas, r.x - 1.0, r.y - 1.0, r.w + 2.0, 1.0, color);
    batch.fill(atlas, r.x - 1.0, r.y + r.h, r.w + 2.0, 1.0, color);
    batch.fill(atlas, r.x - 1.0, r.y, 1.0, r.h, color);
    batch.fill(atlas, r.x + r.w, r.y, 1.0, r.h, color);
}

/// After the chat screen draws: the button at the end of the chat box, and the panel above
/// it while open.
pub(super) fn draw_picker(game: &Game<'_>, batch: &mut HudBatch) {
    if !picker_on(game) {
        with(|s| {
            s.open = false;
            s.layout = None;
        });
        return;
    }
    let layout = Layout::new(batch.width, batch.height);
    let (open, page, recent) = with(|s| {
        s.layout = Some(layout);
        s.page = s.page.min(Page::ALL.len() - 1);
        (s.open, s.page, s.recent.clone())
    });
    let (cx, cy) = game.cursor;
    let atlas = &game.gui.atlas;
    // The shared Sunrise dark look: #0F1118 card, charcoal chips, the live accent.
    let accent = 0xFF00_0000 | crate::game::chatty_card::accent(game);
    let tab_on = crate::game::chatty_card::tab_on(accent);
    // The button: a small raised square with a face on it.
    let b = layout.button;
    let hot = open || b.contains(cx, cy);
    batch.fill(atlas, b.x, b.y, b.w, b.h, if hot { 0xFF_2E3546 } else { 0xE0_1E2330 });
    outline(game, batch, b, if hot { accent } else { 0xFF_2E3546 });
    let face = if drawable(&atlas.font, '☺') { '☺' } else { '#' };
    glyph_in(game, batch, face, b, if hot { 0xFF_FFFFFF } else { 0xFF_E8DCC8 });
    if !open {
        return;
    }
    let p = layout.panel;
    outline(game, batch, p, 0xFF_1E2330);
    batch.fill(atlas, p.x, p.y, p.w, p.h, 0xF4_0F1118);
    batch.fill(atlas, p.x, p.y, p.w, 2.0, accent);
    let mut hovered_page = None;
    for (i, pg) in Page::ALL.iter().enumerate() {
        let r = layout.tab(i);
        let over = r.contains(cx, cy);
        if over {
            hovered_page = Some(*pg);
        }
        if i == page {
            batch.fill(atlas, r.x, r.y, r.w, r.h, tab_on);
        } else if over {
            batch.fill(atlas, r.x, r.y, r.w, r.h, 0xFF_1E2330);
        }
        let color = if i == page { 0xFF_FFFFFF } else { 0xFF_8B93A7 };
        glyph_in(game, batch, pg.icon(), r, color);
    }
    batch.fill(atlas, p.x + 3.0, layout.grid.1 - 2.0, p.w - 6.0, 1.0, 0xFF_1E2330);
    let shown = Page::ALL[page];
    let list = symbols_of(game, shown, &recent);
    for (i, c) in list.iter().enumerate() {
        let r = layout.cell(i);
        if r.contains(cx, cy) {
            batch.fill(atlas, r.x, r.y, r.w, r.h, 0xFF_2E3546);
        }
        glyph_in(game, batch, *c, r, 0xFF_FFFFFF);
    }
    // The page's name above the panel (the one under the cursor while over a tab).
    let name = hovered_page.unwrap_or(shown).label();
    let mut draws = vec![sunrise_client::hud_layout::GuiDraw::Text {
        text: Text::translate(name, &[]),
        x: p.x as i32,
        y: (p.y - 11.0) as i32,
        color: 0xFF_E8DCC8,
        shadow: true,
    }];
    if list.is_empty() {
        draws.push(sunrise_client::hud_layout::GuiDraw::Text {
            text: Text::translate("chatty.symbols.none_yet", &[]),
            x: (layout.grid.0 + 2.0) as i32,
            y: (layout.grid.1 + 2.0) as i32,
            color: 0xFF_8B93A7,
            shadow: false,
        });
    }
    sunrise_client::hud_layout::paint(batch, atlas, &draws);
}

/// A left click on the chat screen: true when the picker took it.
pub(super) fn click(game: &mut Game<'_>) -> bool {
    if !picker_on(game) {
        return false;
    }
    let Some(layout) = with(|s| s.layout) else {
        return false;
    };
    let (x, y) = game.cursor;
    let open = with(|s| s.open);
    match layout.hit(open, x, y) {
        Hit::Button => {
            with(|s| s.open = !s.open);
            true
        }
        Hit::Tab(i) => {
            with(|s| s.page = i.min(Page::ALL.len() - 1));
            true
        }
        Hit::Cell(i) => {
            let (page, recent) = with(|s| (s.page.min(Page::ALL.len() - 1), s.recent.clone()));
            let list = symbols_of(game, Page::ALL[page], &recent);
            if let Some(&c) = list.get(i) {
                game.chat_type(&c.to_string());
                with(|s| symbols::remember(&mut s.recent, c));
            }
            true
        }
        Hit::Panel => true,
        Hit::Outside => {
            with(|s| s.open = false);
            false
        }
    }
}

/// A key on the chat screen: Escape closes the open picker first.
pub(super) fn key(game: &Game<'_>, key: winit::keyboard::KeyCode) -> bool {
    if key != winit::keyboard::KeyCode::Escape || !picker_on(game) {
        return false;
    }
    with(|s| std::mem::replace(&mut s.open, false))
}
