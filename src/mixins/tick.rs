// Once a client tick: joining and leaving, the player list's names, writing the history.
fn chatty_tick(game: &mut crate::game::Game<'_>) {
    crate::game::chatty_glue::tick(game);
}
