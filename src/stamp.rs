//! The time in front of a message.

use crate::config::Clock;

/// `[2:05 PM] ` or `[14:05] `, with `:09` seconds when asked for.
pub fn short(clock: Clock, seconds: bool, hour: u32, minute: u32, second: u32) -> String {
    let secs = if seconds { format!(":{second:02}") } else { String::new() };
    match clock {
        Clock::Twelve => {
            let half = if hour < 12 { "AM" } else { "PM" };
            let h = match hour % 12 {
                0 => 12,
                h => h,
            };
            format!("[{h}:{minute:02}{secs} {half}] ")
        }
        Clock::TwentyFour => format!("[{hour:02}:{minute:02}{secs}] "),
    }
}

/// The hover over the time: the whole date and time, `Wednesday 18 November 2026, 2:05:09 PM`.
pub fn long(clock: Clock, date: &str, hour: u32, minute: u32, second: u32) -> String {
    match clock {
        Clock::Twelve => {
            let half = if hour < 12 { "AM" } else { "PM" };
            let h = match hour % 12 {
                0 => 12,
                h => h,
            };
            format!("{date}, {h}:{minute:02}:{second:02} {half}")
        }
        Clock::TwentyFour => format!("{date}, {hour:02}:{minute:02}:{second:02}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_clocks() {
        assert_eq!(short(Clock::Twelve, false, 0, 5, 0), "[12:05 AM] ");
        assert_eq!(short(Clock::Twelve, false, 12, 0, 0), "[12:00 PM] ");
        assert_eq!(short(Clock::Twelve, true, 14, 5, 9), "[2:05:09 PM] ");
        assert_eq!(short(Clock::TwentyFour, false, 9, 7, 0), "[09:07] ");
        assert_eq!(short(Clock::TwentyFour, true, 23, 59, 1), "[23:59:01] ");
        assert_eq!(long(Clock::Twelve, "Mon 1 June 2026", 13, 2, 3), "Mon 1 June 2026, 1:02:03 PM");
    }
}
