// After the chat screen draws: the symbol button and its panel.
fn chatty_draw_screen(
    value: (),
    game: &mut crate::game::Game<'_>,
    batch: &mut sunrise_client::hud::HudBatch,
) {
    let () = value;
    crate::game::chatty_glue::draw_picker(game, batch);
}
