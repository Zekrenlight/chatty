// A setting changed on a mod's page: Chatty's clear buttons.
fn chatty_setting_changed(game: &crate::game::Game<'_>, mod_id: &str, key: &str) {
    crate::game::chatty_glue::setting_changed(game, mod_id, key);
}
