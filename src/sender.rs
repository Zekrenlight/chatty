//! Who sent a chat line, read from its text alone.
//!
//! Servers dress chat up in many ways: the game's own `<Name> hello`, `Name: hello`,
//! `[Admin] Name » hello`, `Name whispers to you: hello`, `* Name waves`. Chatty looks only
//! at the part before the first separator, and only takes a word there that is a player the
//! game knows (the player list), so a line such as `Server: restarting` gets no head.

/// The sender found on a line: the name, and the character where a head may go.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sender {
    /// Index (in characters) of the name, or of the `<` / `*` in front of it.
    pub start: usize,
    /// Index of the first character that isn't blank.
    pub line_start: usize,
    pub name: String,
}

/// How far into a line a separator may be.
const SEARCH: usize = 72;
/// How many separators are tried before giving up.
const TRIES: usize = 4;

/// A character a Minecraft name may hold.
fn name_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// A whole word that could be a Minecraft name.
pub fn valid_name(word: &str) -> bool {
    (2..=16).contains(&word.len()) && word.chars().all(name_char)
}

/// A name taken without the player list: it needs a letter, so `12` of `[12:00]` isn't one.
fn loose_name(word: &str) -> bool {
    valid_name(word) && word.chars().any(|c| c.is_ascii_alphabetic())
}

/// Characters that end a word before a name: blanks and brackets of every kind.
fn breaks(c: char) -> bool {
    c.is_whitespace() || "[](){}<>|".contains(c)
}

/// Characters after which the message starts.
fn separator(c: char) -> bool {
    matches!(c, ':' | '»' | '>' | '→' | '➜' | '➤' | '⇒' | '›' | '⋙' | '┃')
}

/// The words of `chars[from..to]` with their start, trimmed of marks a server puts round a
/// name (`~Nick`, `Name*`, `@Name`).
fn words(chars: &[char], from: usize, to: usize) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    let mut i = from;
    while i < to {
        if breaks(chars[i]) {
            i += 1;
            continue;
        }
        let start = i;
        while i < to && !breaks(chars[i]) {
            i += 1;
        }
        let mut a = start;
        let mut b = i;
        while a < b && !name_char(chars[a]) {
            a += 1;
        }
        while b > a && !name_char(chars[b - 1]) {
            b -= 1;
        }
        if a < b {
            out.push((a, chars[a..b].iter().collect()));
        }
    }
    out
}

/// The sender of the line `chars`. `known` says whether a name belongs to a player the game
/// knows. With `known_only` off, a single word in front of a separator is taken even when no
/// listed player has that name.
pub fn find(chars: &[char], known: &dyn Fn(&str) -> bool, known_only: bool) -> Option<Sender> {
    let lead = chars.iter().position(|c| !c.is_whitespace())?;
    let end = chars.len().min(lead + SEARCH);
    let found = |start: usize, name: String| Sender {
        start,
        line_start: lead,
        name,
    };
    // The game's own form, `<Name> message` (a rank may sit inside: `<[VIP] Name>`).
    if chars[lead] == '<'
        && let Some(close) = (lead + 1..end).find(|&i| chars[i] == '>')
    {
        let inside = words(chars, lead + 1, close);
        if let Some((_, name)) = inside.iter().rev().find(|(_, w)| valid_name(w) && known(w)) {
            return Some(found(lead, name.clone()));
        }
        if let Some((_, name)) = inside.last().filter(|(_, w)| loose_name(w)) {
            return Some(found(lead, name.clone()));
        }
    }
    // An action, `* Name waves`.
    if chars[lead] == '*' && chars.get(lead + 1) == Some(&' ') {
        if let Some((at, name)) = words(chars, lead + 2, end).into_iter().next()
            && valid_name(&name)
            && (known(&name) || !known_only)
        {
            return Some(found(at, name));
        }
    }
    // `[Rank] Name: message`, `Name » message`, `Name whispers to you: message`.
    let mut tries = 0;
    for i in lead..end {
        if !separator(chars[i]) {
            continue;
        }
        tries += 1;
        if tries > TRIES {
            break;
        }
        let before = words(chars, lead, i);
        if let Some((at, name)) = before
            .iter()
            .rev()
            .take(5)
            .find(|(_, w)| valid_name(w) && known(w))
        {
            return Some(found(*at, name.clone()));
        }
        if !known_only
            && before.len() <= 3
            && let Some((at, name)) = before.last().filter(|(_, w)| loose_name(w))
        {
            return Some(found(*at, name.clone()));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(line: &str, known_only: bool) -> Option<(usize, String)> {
        let chars: Vec<char> = line.chars().collect();
        let known = |w: &str| ["Zekren", "Steve_2", "Alex"].iter().any(|k| k.eq_ignore_ascii_case(w));
        find(&chars, &known, known_only).map(|s| (s.start, s.name))
    }

    #[test]
    fn the_games_own_form() {
        assert_eq!(run("<Zekren> hi", true), Some((0, "Zekren".into())));
        // Unknown names in the game's own form are still names.
        assert_eq!(run("<Somebody> hi", true), Some((0, "Somebody".into())));
        assert_eq!(run("<[VIP] Alex> hi", true), Some((0, "Alex".into())));
    }

    #[test]
    fn ranks_and_separators() {
        assert_eq!(run("[Admin] Zekren: hello", true), Some((8, "Zekren".into())));
        assert_eq!(run("Alex » hi there", true), Some((0, "Alex".into())));
        assert_eq!(run("[Member] ~Steve_2 > yo", true), Some((10, "Steve_2".into())));
        assert_eq!(run("Zekren whispers to you: psst", true), Some((0, "Zekren".into())));
        assert_eq!(run("[12:00] Alex: hi", true), Some((8, "Alex".into())));
        assert_eq!(run("* Alex waves", true), Some((2, "Alex".into())));
    }

    #[test]
    fn lines_without_a_known_sender() {
        assert_eq!(run("Server: restarting in 5 minutes", true), None);
        assert_eq!(run("Alex joined the game", true), None);
        assert_eq!(run("Visit https://example.com", true), None);
        assert_eq!(run("", true), None);
        // With known_only off, one word before a separator is taken.
        assert_eq!(run("Server: restarting", false), Some((0, "Server".into())));
        assert_eq!(run("A long line of words here: no", false), None);
    }
}
