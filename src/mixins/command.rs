// `/chatty`, `/chatty clear` and `/chatty clear all`.
fn chatty_command(
    game: &mut crate::game::Game<'_>,
    command: &str,
    ran: &mut bool,
) -> ::core::ops::ControlFlow<()> {
    crate::game::chatty_glue::command(game, command, ran);
    if *ran {
        ::core::ops::ControlFlow::Break(())
    } else {
        ::core::ops::ControlFlow::Continue(())
    }
}
