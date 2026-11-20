//! Where a server's or world's chat history is kept.
//!
//! Each server (by address) and each single player world (by save folder) has its own file
//! under `history/` in Chatty's data folder, so clearing one leaves the rest alone.

/// The file name for the history of `key` (`server:play.example.com`, `world:New World`):
/// readable, safe on every system, and told apart by a short hash.
pub fn file_name(key: &str) -> String {
    let mut readable: String = key
        .chars()
        .map(|c| {
            let c = c.to_ascii_lowercase();
            if c.is_ascii_alphanumeric() || c == '.' || c == '-' { c } else { '_' }
        })
        .collect();
    readable.truncate(48);
    // FNV-1a: stable across runs and builds.
    let mut hash: u32 = 0x811c_9dc5;
    for b in key.bytes() {
        hash ^= u32::from(b);
        hash = hash.wrapping_mul(0x0100_0193);
    }
    format!("{readable}-{hash:08x}.json")
}

/// The history folder's files are Chatty's own when they end so.
pub fn is_history_file(name: &str) -> bool {
    name.ends_with(".json") && !name.starts_with('.')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_safe_and_distinct() {
        let a = file_name("server:Play.Example.com:25565");
        assert!(a.starts_with("server_play.example.com_25565-"));
        assert!(a.ends_with(".json"));
        assert_ne!(file_name("world:A/B"), file_name("world:A_B"));
        assert!(!file_name("world:../../x").contains('/'));
        assert_eq!(file_name("world:x"), file_name("world:x"));
    }
}
