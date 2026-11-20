//! The symbol picker: its pages of plain Unicode characters and where its parts sit.
//!
//! Every symbol is an ordinary character anyone's game can show; the picker only types it
//! into the chat box. Characters the game's font can't draw are left out when it opens.

/// A page of the picker.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page {
    /// The symbols used last, this session.
    Recent,
    Faces,
    Weather,
    Arrows,
    Shapes,
    Gear,
    Marks,
    Letters,
    Emoji,
}

impl Page {
    pub const ALL: [Page; 9] = [
        Page::Recent,
        Page::Faces,
        Page::Weather,
        Page::Arrows,
        Page::Shapes,
        Page::Gear,
        Page::Marks,
        Page::Letters,
        Page::Emoji,
    ];

    /// The character on the page's tab.
    pub fn icon(self) -> char {
        match self {
            Page::Recent => '⌚',
            Page::Faces => '☺',
            Page::Weather => '☀',
            Page::Arrows => '➜',
            Page::Shapes => '◆',
            Page::Gear => '⚔',
            Page::Marks => '✔',
            Page::Letters => 'Ω',
            Page::Emoji => '★',
        }
    }

    /// The page's name, a translation key (the tab's tooltip).
    pub fn label(self) -> &'static str {
        match self {
            Page::Recent => "chatty.symbols.recent",
            Page::Faces => "chatty.symbols.faces",
            Page::Weather => "chatty.symbols.weather",
            Page::Arrows => "chatty.symbols.arrows",
            Page::Shapes => "chatty.symbols.shapes",
            Page::Gear => "chatty.symbols.gear",
            Page::Marks => "chatty.symbols.marks",
            Page::Letters => "chatty.symbols.letters",
            Page::Emoji => "chatty.symbols.emoji",
        }
    }

    /// The page's characters (none for Recent, which the game half fills).
    pub fn symbols(self) -> &'static str {
        match self {
            Page::Recent => "",
            Page::Faces => "☺☻☹♥♡❤❥❣❦❧✿❀❁✾✽❃❋♪♫♬♩♭♮♯☮☯✌☝✍☚☛☜☞♀♂⚢⚣⚤",
            Page::Weather => "☀☼☁☂☔☃⛄❄❅❆☄★☆✦✧✩✪✫✬✭✮✯✰⁂☽☾☘⚘⚡♨⛅⛈",
            Page::Arrows => "←↑→↓↔↕↖↗↘↙⇐⇑⇒⇓⇔⇕⇦⇧⇨⇩➔➜➝➞➟➠➡➢➣➤➥➦↩↪↺↻⟲⟳▲▼◀▶△▽◁▷«»‹›",
            Page::Shapes => "■□▢▣▤▥▦▧▨▩▪▫▬▭▮▯●○◎◉◌◍◐◑◒◓◆◇◈◊♦♢░▒▓█▀▄▌▐─│┼═║╬╔╗╚╝",
            Page::Gear => "⚔⚒⛏⚓⚑⚐☠☢☣⚠⚙⚛⚗⚖⌛⏳☎✉✂✎✏✒♔♕♖♗♘♙♚♛♜♝♞♟♠♣♤♧⚀⚁⚂⚃⚄⚅⛨☕⌘⏏",
            Page::Marks => "✓✔✗✘☐☑☒±×÷≈≠≤≥∞√∑∏∆∇∫∴∵∅∈∩∪°‰¹²³½¼¾©®™¶†‡•·…¡¿№",
            Page::Letters => "αβγδεζηθικλμνξπρστυφχψωΓΔΘΛΞΠΣΦΨΩ$¢£¥€₹₽₩₿ÆæØøÅåÐðÞþßŒœ",
            Page::Emoji => "😀😃😄😁😆😅😂🙂😉😊😇😍😘😋😛😜🤔😐😑😶🙄😏😴😎🤓😮😲😢😭😤😡💀👍👎👋👏🙏💪👀🎉🎂🎁🔥💯💤💬❗❓🌈🍕🍪🐱🐶🦊🐝🌸🌙",
        }
    }
}

/// How many symbols the picker shows in a row, and rows.
pub const COLS: usize = 10;
pub const ROWS: usize = 6;
/// A cell's side, in GUI pixels.
pub const CELL: f32 = 12.0;
/// The button's side.
pub const BUTTON: f32 = 12.0;
const PAD: f32 = 3.0;

/// A rectangle in GUI pixels: x, y, width, height.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x && x < self.x + self.w && y >= self.y && y < self.y + self.h
    }
}

/// Where the picker's parts sit on a screen `width` x `height` GUI pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Layout {
    /// The button at the right end of the chat box.
    pub button: Rect,
    /// The panel above it.
    pub panel: Rect,
    /// The top-left of the first tab.
    pub tabs: (f32, f32),
    /// The top-left of the first cell.
    pub grid: (f32, f32),
}

impl Layout {
    pub fn new(width: f32, height: f32) -> Self {
        // The chat box runs from 2 to width - 2, from height - 14 to height - 2.
        let button = Rect {
            x: width - 2.0 - BUTTON,
            y: height - 14.0,
            w: BUTTON,
            h: BUTTON,
        };
        let w = COLS as f32 * CELL + 2.0 * PAD;
        let h = PAD + CELL + PAD + ROWS as f32 * CELL + PAD;
        let panel = Rect {
            x: (width - 2.0 - w).max(2.0),
            y: (height - 16.0 - h).max(2.0),
            w,
            h,
        };
        Self {
            button,
            panel,
            tabs: (panel.x + PAD, panel.y + PAD),
            grid: (panel.x + PAD, panel.y + PAD + CELL + PAD),
        }
    }

    pub fn tab(&self, i: usize) -> Rect {
        Rect {
            x: self.tabs.0 + i as f32 * CELL,
            y: self.tabs.1,
            w: CELL,
            h: CELL,
        }
    }

    pub fn cell(&self, i: usize) -> Rect {
        Rect {
            x: self.grid.0 + (i % COLS) as f32 * CELL,
            y: self.grid.1 + (i / COLS) as f32 * CELL,
            w: CELL,
            h: CELL,
        }
    }

    /// What a click at (`x`, `y`) lands on, the panel being `open`.
    pub fn hit(&self, open: bool, x: f32, y: f32) -> Hit {
        if self.button.contains(x, y) {
            return Hit::Button;
        }
        if !open || !self.panel.contains(x, y) {
            return Hit::Outside;
        }
        if let Some(i) = (0..Page::ALL.len()).find(|&i| self.tab(i).contains(x, y)) {
            return Hit::Tab(i);
        }
        if let Some(i) = (0..COLS * ROWS).find(|&i| self.cell(i).contains(x, y)) {
            return Hit::Cell(i);
        }
        Hit::Panel
    }
}

/// What a click landed on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hit {
    Button,
    Tab(usize),
    Cell(usize),
    /// The panel, between its parts.
    Panel,
    Outside,
}

/// Puts `symbol` first in the recent list, keeping one page of them.
pub fn remember(recent: &mut Vec<char>, symbol: char) {
    recent.retain(|&c| c != symbol);
    recent.insert(0, symbol);
    recent.truncate(COLS * ROWS);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_page_fits_and_types_nothing_forbidden() {
        for page in Page::ALL {
            let n = page.symbols().chars().count();
            assert!(n <= COLS * ROWS, "{page:?} has {n}");
            assert!(!page.symbols().contains('§'));
            assert!(page.symbols().chars().all(|c| !c.is_control() && !c.is_whitespace()));
        }
        assert!(Page::ALL.len() as f32 * CELL <= COLS as f32 * CELL);
    }

    #[test]
    fn clicks_land_where_drawn() {
        let l = Layout::new(400.0, 240.0);
        assert_eq!(l.hit(false, l.button.x + 1.0, l.button.y + 1.0), Hit::Button);
        assert_eq!(l.hit(false, l.panel.x + 5.0, l.panel.y + 5.0), Hit::Outside);
        let t = l.tab(2);
        assert_eq!(l.hit(true, t.x + 1.0, t.y + 1.0), Hit::Tab(2));
        let c = l.cell(13);
        assert_eq!(l.hit(true, c.x + 6.0, c.y + 6.0), Hit::Cell(13));
        assert_eq!(l.hit(true, 5.0, 5.0), Hit::Outside);
        assert!(l.panel.y + l.panel.h <= l.button.y);
    }

    #[test]
    fn recent_keeps_the_newest_first() {
        let mut r = vec!['a', 'b'];
        remember(&mut r, 'b');
        assert_eq!(r, vec!['b', 'a']);
    }
}
