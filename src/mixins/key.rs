// A key on the chat screen: Escape closes the open symbol picker before the chat.
fn chatty_key(
    game: &mut crate::game::Game<'_>,
    key: winit::keyboard::KeyCode,
) -> ::core::ops::ControlFlow<()> {
    if crate::game::chatty_glue::key(game, key) {
        ::core::ops::ControlFlow::Break(())
    } else {
        ::core::ops::ControlFlow::Continue(())
    }
}
