// PvP & HUD's chat options as the chat reads them: Chatty's repeats and timestamps take over.
fn chatty_chat_options(options: ChatOptions, game: &crate::game::Game<'_>) -> ChatOptions {
    crate::game::chatty_glue::qol(game, options)
}
