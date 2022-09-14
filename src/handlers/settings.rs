use loon::Opts;
use teloxide::{
    dispatching::{HandlerExt, UpdateFilterExt},
    payloads::SendMessageSetters,
    requests::{Request, Requester},
    types::{Message, ParseMode, Update},
    utils::{command::BotCommands, markdown::escape as markdown_escape},
};

use crate::database::Database;
use crate::types::{AtomicHandler, DeleteIttBot, HandlerResult, Locale, Localization};

use super::filters::is_privileged;
use super::utils::{delete_message, get_locale};

#[derive(BotCommands, Clone)]
#[command(rename = "snake_case")]
enum GroupCmd {
    #[command()]
    Help,

    #[command()]
    VoteCount { count: i64 },

    #[command()]
    Language { lang: String },

    #[command()]
    Languages,
}

#[derive(BotCommands, Clone)]
#[command(rename = "snake_case")]
enum PersonalCmd {
    #[command()]
    Start,
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
        .parse_mode(ParseMode::MarkdownV2)
        .await?;

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

        bot.send_message(msg.chat.id, response).await?;
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

async fn group_handler(
    bot: DeleteIttBot,
    msg: Message,
    command: GroupCmd,
    db: Database,
    loc: Localization,
    locales: Vec<Locale>,
) -> HandlerResult {
    match command {
        GroupCmd::Help => help_handler(&bot, &msg, &db, &loc).await,
        GroupCmd::VoteCount { count } => votes_count_handler(&bot, &msg, &db, &loc, count).await,
        GroupCmd::Language { lang } => {
            language_handler(&bot, &msg, &db, &loc, lang, &locales).await
        }
        GroupCmd::Languages => languages_handler(&bot, &msg, &db, &loc, &locales).await,
    }
}

async fn start_handler(bot: &DeleteIttBot, msg: &Message) -> HandlerResult {
    let me = bot.get_me().send().await?;

    let start_msg = markdown_escape(&format!(
        "Hello! I'm {}. I can help you keep your chats clean. Mention me (@{}) in reply to \
        the message you want to delete. I will then set up a poll, which is used to take a \
        decision. These decision parameters can be configured on a per-chat basis. See \
        /help (in a group). \n\n\
        My code is free and open source, hosted at https://github.com/ssiyad/delete-itt.",
        me.full_name(),
        me.username()
    ));

    bot.send_message(msg.chat.id, start_msg)
        .disable_web_page_preview(true)
        .parse_mode(ParseMode::MarkdownV2)
        .await?;

    Ok(())
}

async fn peronal_handler(bot: DeleteIttBot, msg: Message, command: PersonalCmd) -> HandlerResult {
    match command {
        PersonalCmd::Start => start_handler(&bot, &msg).await,
    }
}

pub fn settings_handler() -> AtomicHandler {
    let private_commands = Update::filter_message()
        .filter(|msg: Message| msg.chat.is_private())
        .filter_command::<PersonalCmd>()
        .endpoint(peronal_handler);

    let group_commands = Update::filter_message()
        .filter(|msg: Message| msg.chat.is_group() || msg.chat.is_supergroup())
        .filter_command::<GroupCmd>()
        .filter_async(is_privileged)
        .map_async(delete_message)
        .endpoint(group_handler);

    teloxide::dptree::entry()
        .branch(private_commands)
        .branch(group_commands)
}
