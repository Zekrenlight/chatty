// SPDX-License-Identifier: MIT
// Dragging on Chatty's settings card: its sliders and scrollbar.
fn chatty_card_options_drag(game: &mut crate::game::Game<'_>) -> ::core::ops::ControlFlow<Result<()>> {
    if !crate::game::chatty_card::active(game) {
        return ::core::ops::ControlFlow::Continue(());
    }
    ::core::ops::ControlFlow::Break(crate::game::chatty_card::drag(game))
}
