// A left click on the chat screen: the symbol picker first.
fn chatty_click(game: &mut crate::game::Game<'_>) -> ::core::ops::ControlFlow<bool> {
    if crate::game::chatty_glue::click(game) {
        ::core::ops::ControlFlow::Break(true)
    } else {
        ::core::ops::ControlFlow::Continue(())
    }
}
