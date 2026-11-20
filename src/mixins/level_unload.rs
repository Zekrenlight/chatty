// A level is going: the history is written now.
fn chatty_level_unload(game: &crate::game::Game<'_>) {
    crate::game::chatty_glue::level_unload(game);
}
