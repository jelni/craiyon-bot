use std::sync::Arc;
use std::time::{Duration, Instant};

use super::command_context::CommandContext;
use super::command_manager::CommandInstance;
use crate::bot::TdResult;
use crate::commands::CommandError;
use crate::utilities::text_utils;

pub async fn dispatch_command(
    command: Arc<CommandInstance>,
    arguments: String,
    context: CommandContext,
) {
    if let Some(cooldown) = check_rate_limit(&command, &context) {
        if let Err(err) = Box::pin(report_rate_limit(&context, cooldown)).await {
            log::error!(
                "TDLib error occurred while reporting a rate limit: {}: {}",
                err.code,
                err.message
            );
        }
        return;
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
        CommandError::Custom(text) => context.reply(text).await?,
        CommandError::CustomFormattedText(text) => context.reply_formatted_text(text).await?,
        CommandError::ArgumentConversion(err) => context.reply(err.to_string()).await?,
        CommandError::Telegram(err) => {
            log::error!("TDLib error in the {command} command: {}: {}", err.code, err.message);
            context.reply(format!("sending the message failed ({}) 😔", err.message)).await?
        }
        CommandError::Server(status_code) => {
            context
                .reply(format!(
                "an external service used by this command is currently offline ({status_code})."
            ))
                .await?
        }
        CommandError::Reqwest(err) => {
            log::error!("HTTP error in the {command} command: {err}");
            context.reply(err.without_url().to_string()).await?
        }
    };

    Ok(())
}
