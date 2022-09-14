use loon::Opts;
use teloxide::{
    dispatching::{HandlerExt, UpdateFilterExt},
    payloads::SendMessageSetters,
    requests::Requester,
    types::{Message, ParseMode, Update},
    utils::{command::BotCommands, markdown::escape as markdown_escape},
};

use crate::database::Database;
use crate::types::{AtomicHandler, DeleteIttBot, HandlerResult, Locale, Localization};

use super::filters::is_privileged;
use super::utils::{delete_message, get_locale};

#[derive(BotCommands, Clone)]
#[command(rename = "snake_case")]
enum Cmd {
    #[command()]
    Help,

    #[command()]
    VoteCount { count: i64 },

    #[command()]
    Language { lang: String },

    #[command()]
    Languages,
}

fn format_help_command<S, T>(locale: S, command: T, loc: &Localization) -> String
where
    S: Into<String>,
    T: Into<String> + Clone,
{
    let response = loc
        .t(
            format!("help.commands.{}", command.clone().into()).as_str(),
            Opts::default().locale(&locale.into()),
        )
        .unwrap_or_else(|_| "".into());

    format!(
        "/{} — _{}_",
        markdown_escape(&command.into()),
        markdown_escape(&response)
    )
}

async fn help_handler(
    bot: &DeleteIttBot,
    msg: &Message,
    db: &Database,
    loc: &Localization,
) -> HandlerResult {
    let chat_id = msg.chat.id.0;
    let locale = &get_locale(db, chat_id).await;

    let response = vec!["help", "vote_count", "language", "languages"]
        .into_iter()
        .map(|s| format_help_command(locale, s, loc))
        .collect::<Vec<String>>()
        .join("\n");

    bot.send_message(msg.chat.id, response)
        .reply_to_message_id(msg.id)
        .parse_mode(ParseMode::MarkdownV2)
        .await
        .unwrap();

    Ok(())
}

async fn votes_count_handler(
    bot: &DeleteIttBot,
    msg: &Message,
    db: &Database,
    loc: &Localization,
    count: i64,
) -> HandlerResult {
    if count <= 0 || count > 10 {
        bot.send_message(msg.chat.id, "Count must be in range of 1 to 10")
            .reply_to_message_id(msg.id)
            .await?;

        return Ok(());
    }

    let chat_id = msg.chat.id.0;

    if let Ok(None) = db.get_chat(chat_id).await {
        db.create_chat(chat_id).await?;
    }

    if let Ok(true) = db.set_chat_votes(chat_id, count).await {
        let response = loc.t(
            "vote_count.updated",
            Opts::default()
                .var("count", count)
                .locale(&get_locale(db, chat_id).await),
        )?;

        bot.send_message(msg.chat.id, response)
            .reply_to_message_id(msg.id)
            .await?;
    }

    Ok(())
}

async fn language_handler(
    bot: &DeleteIttBot,
    msg: &Message,
    db: &Database,
    loc: &Localization,
    lang: Locale,
    locales: &[Locale],
) -> HandlerResult {
    if !locales.contains(&lang) {
        bot.send_message(msg.chat.id, format!("Invalid language: {}", lang))
            .await?;

        return Ok(());
    }

    let chat_id = msg.chat.id.0;

    if let Ok(None) = db.get_chat(chat_id).await {
        db.create_chat(chat_id).await?;
    }

    if let Ok(true) = db.set_chat_locale(chat_id, &lang).await {
        let response = loc.t(
            "language.updated",
            Opts::default()
                .var("language", lang)
                .locale(&get_locale(db, chat_id).await),
        )?;

        bot.send_message(msg.chat.id, response)
            .parse_mode(ParseMode::MarkdownV2)
            .await?;
    }

    Ok(())
}

async fn languages_handler(
    bot: &DeleteIttBot,
    msg: &Message,
    db: &Database,
    loc: &Localization,
    locales: &[Locale],
) -> HandlerResult {
    let title = loc.t(
        "language.list_title",
        Opts::default().locale(&get_locale(db, msg.chat.id.0).await),
    )?;

    let response = format!("*{}*\n{}", title, locales.join(" "));

    bot.send_message(msg.chat.id, response)
        .parse_mode(ParseMode::MarkdownV2)
        .await?;

    Ok(())
}

async fn handler(
    bot: DeleteIttBot,
    msg: Message,
    command: Cmd,
    db: Database,
    loc: Localization,
    locales: Vec<Locale>,
) -> HandlerResult {
    match command {
        Cmd::Help => help_handler(&bot, &msg, &db, &loc).await,
        Cmd::VoteCount { count } => votes_count_handler(&bot, &msg, &db, &loc, count).await,
        Cmd::Language { lang } => language_handler(&bot, &msg, &db, &loc, lang, &locales).await,
        Cmd::Languages => languages_handler(&bot, &msg, &db, &loc, &locales).await,
    }
}

pub fn settings_handler() -> AtomicHandler {
    Update::filter_message()
        .filter_command::<Cmd>()
        .filter_async(is_privileged)
        .map_async(handler)
        .endpoint(delete_message)
}
