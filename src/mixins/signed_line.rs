// A line joins the connection's chat: Chatty's heads, timestamp and repeat count go on it, and
// it is kept for the history. The rest is the game's own (the Chat option's filter, the tag of
// a player message's trust, the connection's record of recent lines).
fn chatty_signed_line(
    game: &mut crate::game::Game<'_>,
    content: &sunrise_client::text::Text,
    source: sunrise_client::chat::ChatSource,
    signature: &Option<Signature>,
    tag: &Option<sunrise_client::chat::Tag>,
) -> ::core::ops::ControlFlow<()> {
    if !game.ui.chat.shows(source) {
        return ::core::ops::ControlFlow::Continue(());
    }
    let Some(shown) =
        crate::game::chatty_glue::incoming(game, content, source, signature.is_some())
    else {
        return ::core::ops::ControlFlow::Continue(());
    };
    let plain = content.plain();
    let focused = game.screen == Screen::Chat;
    let ticks = game.visual_ticks();
    if source == sunrise_client::chat::ChatSource::Player {
        game.ui
            .chat
            .add_signed(&game.gui.atlas, shown, *signature, tag.clone(), ticks, focused);
    } else {
        game.ui.chat.add(
            &game.gui.atlas,
            shown,
            Some(sunrise_client::chat::SYSTEM_INDICATOR),
            source,
            ticks,
            focused,
        );
    }
    crate::game::chatty_glue::shown(game);
    if let Some(r) = game.remote.as_mut() {
        r.chat.push((plain, ::std::time::Instant::now()));
        r.chat_received += 1;
        if r.chat.len() > 100 {
            r.chat.remove(0);
        }
    }
    ::core::ops::ControlFlow::Break(())
}
