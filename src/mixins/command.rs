// `/chatty`, `/chatty clear` and `/chatty clear all`.
fn chatty_command(game: &mut crate::game::Game<'_>, line: &str) {
    let mut ran = false;
    crate::game::chatty_glue::command(game, line, &mut ran);
}
