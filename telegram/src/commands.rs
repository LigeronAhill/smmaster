use teloxide::utils::command::BotCommands;

/// Поддерживаются следующие команды:
#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
pub enum Command {
    /// Справка
    Help,
    /// Вызвать меню
    Start,
}
