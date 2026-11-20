// SPDX-License-Identifier: MIT
// The wheel on Chatty's settings card.
fn chatty_card_options_scroll(
    game: &mut crate::game::Game<'_>,
    amount: f32,
) -> ::core::ops::ControlFlow<Result<()>> {
    if !crate::game::chatty_card::active(game) {
        return ::core::ops::ControlFlow::Continue(());
    }
    ::core::ops::ControlFlow::Break(crate::game::chatty_card::scroll(game, amount))
}
