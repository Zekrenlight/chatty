// A line joins the window's own chat (no connection): Chatty's heads, timestamp and repeat
// count go on it, and it is kept for the history.
fn chatty_local_line(
    game: &mut crate::game::Game<'_>,
    text: &sunrise_client::text::Text,
    indicator: Option<u32>,
    source: sunrise_client::chat::ChatSource,
) -> ::core::ops::ControlFlow<()> {
    if !game.ui.chat.shows(source) {
        return ::core::ops::ControlFlow::Continue(());
    }
    let Some(shown) = crate::game::chatty_glue::incoming(game, text, source, false) else {
        return ::core::ops::ControlFlow::Continue(());
    };
    let focused = game.screen == Screen::Chat;
    let ticks = game.visual_ticks();
    game.ui
        .chat
        .add(&game.gui.atlas, shown, indicator, source, ticks, focused);
    crate::game::chatty_glue::shown(game);
    ::core::ops::ControlFlow::Break(())
}
