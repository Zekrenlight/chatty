// SPDX-License-Identifier: MIT
// Chatty's settings page: its card is drawn instead of Sunrise's generic page.
fn chatty_card_options_draw(
    game: &mut crate::game::Game<'_>,
    batch: &mut sunrise_client::hud::HudBatch,
) -> ::core::ops::ControlFlow<()> {
    if !crate::game::chatty_card::active(game) {
        return ::core::ops::ControlFlow::Continue(());
    }
    crate::game::chatty_card::draw(game, batch);
    ::core::ops::ControlFlow::Break(())
}
