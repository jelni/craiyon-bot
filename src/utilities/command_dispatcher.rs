use std::sync::Arc;
use std::time::{Duration, Instant};

use tdlib::{enums, functions};

use super::command_context::CommandContext;
use super::command_manager::CommandInstance;
use super::telegram_utils;
use crate::bot::TdResult;
use crate::commands::CommandError;
use crate::utilities::text_utils;

pub async fn dispatch_command(
    command: Arc<CommandInstance>,
    mut arguments: String,
    context: CommandContext,
) {
    if let Some(cooldown) = check_rate_limit(&command, &context) {
        if let Err(err) = report_rate_limit(&context, cooldown).await {
            log::error!(
                "TDLib error occurred while reporting a rate limit: {}: {}",
                err.code,
                err.message
            );
        }
        return;
    }

    if arguments.is_empty() {
        arguments = get_reply_text(&context).await.unwrap_or_default();
    }

    log::info!("running {command} {:?} for {} in {}", arguments, context.user, context.chat);

    if let Err(err) = command.command.execute(&context, arguments).await {
        if let Err(err) = report_command_error(command, &context, err).await {
            log::error!(
                "TDLib error occurred while handling the previous error: {}: {}",
                err.code,
                err.message
            );
        }
    }
}

fn check_rate_limit(command: &CommandInstance, context: &CommandContext) -> Option<u64> {
    let cooldown = command
        .rate_limiter
        .lock()
        .unwrap()
        .update_rate_limit(context.user.id, context.message.date)?
        .try_into()
        .unwrap();

    log::info!(
        "{command} rate limit exceeded by {} by {}",
        text_utils::format_duration(cooldown),
        context.user
    );

    Some(cooldown)
}

async fn get_reply_text(context: &CommandContext) -> Option<String> {
    if context.message.reply_to_message_id == 0 {
        return None;
    }

    let enums::Message::Message(message) = functions::get_message(
        context.message.reply_in_chat_id,
        context.message.reply_to_message_id,
        context.client_id,
    )
    .await
    .ok()?;

    telegram_utils::get_message_text(&message).map(|text| text.text.clone())
}

async fn report_rate_limit(context: &CommandContext, cooldown: u64) -> TdResult<()> {
    if context
        .rate_limits
        .lock()
        .unwrap()
        .rate_limit_exceeded
        .update_rate_limit(context.user.id, context.message.date)
        .is_some()
    {
        return Ok(());
    }

    let cooldown_end = Instant::now() + Duration::from_secs(cooldown.clamp(5, 60));

    let message = context
        .message_queue
        .wait_for_message(
            context
                .reply(format!(
                    "you can use this command again in {}.",
                    text_utils::format_duration(cooldown)
                ))
                .await?
                .id,
        )
        .await?;

    tokio::time::sleep_until(cooldown_end.into()).await;
    context.delete_message(message.id).await?;

    Ok(())
}

async fn report_command_error(
    command: Arc<CommandInstance>,
    context: &CommandContext,
    error: CommandError,
) -> TdResult<()> {
    match error {
        CommandError::CustomError(text) => context.reply(text).await?,
        CommandError::CustomMarkdownError(text) => context.reply_markdown(text).await?,
        CommandError::ArgumentConversionError(err) => context.reply(err.to_string()).await?,
        CommandError::TelegramError(err) => {
            log::error!("TDLib error in the {command} command: {}: {}", err.code, err.message);
            context.reply(format!("sending the message failed ({}) 😔", err.message)).await?
        }
        CommandError::ServerError(status_code) => {
            context
                .reply(format!(
                "an external service used by this command is currently offline ({status_code})."
            ))
                .await?
        }
        CommandError::ReqwestError(err) => {
            log::error!("HTTP error in the {command} command: {err}");
            context.reply(err.without_url().to_string()).await?
        }
    };

    Ok(())
}
