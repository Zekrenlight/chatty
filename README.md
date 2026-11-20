# Chatty

A Sunrise client mod by Zekrenlight (MIT).
Version 1.0.0 · Minecraft Java 26.3 · Sunrise API 0.2 · Mod SDK `9425651` · MIT licence.

Settings: Options > Mods... > Chatty > Settings. Every part has its own switch.

<img width="653" height="78" alt="image" src="https://github.com/user-attachments/assets/67273672-2946-41ee-9449-d532b3ec0597" />
<img width="258" height="242" alt="image" src="https://github.com/user-attachments/assets/54c0643e-ca93-4993-8911-837bfb412a9e" />


## Chat heads
- Each player's face (with the hat layer) in front of their name on chat lines.
- Works with the game's own `<Name> message` and with server formats: `Name: message`,
  `[Admin] Name » message`, `~Nick > message`, `Name whispers to you: message`, `* Name waves`.
- Only names in the player list (Tab) get a face, so `Server: restarting` stays plain
  (Only Listed Players; turn it off to also take a lone word before a colon).
- Show My Own Head, Head Hat Layer, and Head Position (Before Name or Line Start).
- The face is the game's own text head, so it wraps, fades and scrolls with its line and uses
  the skin the player list shows.

## Timestamps
- The time each message came in, at the start of its line: 12-hour (`[2:05 PM]`) or 24-hour
  (`[14:05]`), with or without seconds, in a colour you pick. Hover for the full date.

## Stack Repeats
- The same message again, right after itself, raises a count (`(x3)`) instead of adding lines.
  Signed player messages never stack, since a server may delete one.

## Chat history
- The chat (up to Messages Kept, 10 to 100) and what you sent are kept when you leave and come
  back when you return. Every server and every single player world has its own history.
- A thin "earlier chat" line separates last time's chat from the new (History Divider).
- Clearing: "This Server's History" and "All Chat History" in the settings (click, then click
  again to confirm), or `/chatty clear` and `/chatty clear all` in chat. Clearing empties the chat
  on screen and the sent messages too; commands stay in the Up-arrow history.
- Files: `sunrise-mods/.sunrise-mod-data/chatty/history/`, one per server or world.

## Symbol picker
- A small button at the right end of the chat box opens pages of symbols: faces and hearts,
  sky and stars, arrows, shapes and lines, tools and games, marks and maths, letters and money,
  emoji, and the ones you used recently. Click one to type it; Escape closes the picker.
- All of them are ordinary Unicode characters. Ones the game's font can't draw are left out.

## Notes
- With Stack Repeats on, Sunrise's own PvP & HUD "compact chat" is used to stack; with Chatty's
  timestamps on, PvP & HUD's chat timestamps are held back so lines don't get two.
- A few lines other mods add straight into the chat go in as they are, without a timestamp or
  head, and aren't kept.

## Look and settings

Chatty uses the shared Sunrise dark look: a `#0F1118` card with a 2 px accent edge, tabs and
charcoal chips. The accent is Sunrise orange `#F9801D` at first; change it with **Accent colour**
in the settings card (eight presets, or Reset).
The symbol picker uses the same card and accent.

The settings card opens from SunnyMods' **Settings** button, from Sunrise's own Mods list, or
from the mod's settings key. **Key binds › Rebind** switches to Sunrise's classic page to change keys.

## Compatibility

| Piece | Pin |
| --- | --- |
| Minecraft Java | 26.3 |
| Sunrise API | 0.2 |
| Mod SDK (`[native].sunrise`) | `94256510f74e6843754e0a93fd9e4da7e45b1056` (`9425651`, 19 Nov 2026) |
| Mixins (`[native].mixins`) | `e59da9094f0c2fdda6c627fe9eff22dec6a906f0` |

Chatty works on its own and alongside the other Sunrise mods by Zekrenlight. Its settings-card
hooks act only on its own Options page, so several mods can be installed together without any
conflicting replacements. A mod with native hooks works only on the exact SDK build it was made for.

## Building

```sh
cargo sunrise-mod           # fetches the pinned SDK into .sunrise/ (gitignored)
cargo sunrise-mod check     # type-checks against the pinned SDK
cargo test --lib            # unit tests, no game needed
cargo sunrise-mod build     # target/sunrise-mod/chatty.smod
```

To move to a newer SDK, run `cargo sunrise-mod update --sunrise <build>` (it rewrites the
`sunrise` and `mixins` lines in `sunrise-mod.toml`), then check and rebuild.

Install the `.smod` with sunrise-launcher (**Add from file**), or point **Add from GitHub** at
this repository. Native mods run with full access to your computer; read the source first.

## License

MIT © 2026 Zekrenlight. See [LICENSE](LICENSE).
