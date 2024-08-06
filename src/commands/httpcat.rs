use async_trait::async_trait;

use super::{CommandError, CommandResult, CommandTrait};
use crate::utilities::command_context::CommandContext;
use crate::utilities::convert_argument::{ConvertArgument, StringGreedyOrReply};

pub struct Httpcat;

#[async_trait]
impl CommandTrait for Httpcat {
    fn command_names(&self) -> &[&str] {
        &["httpcat", "cat"]
    }

    async fn execute(&self, ctx: &CommandContext, arguments: String) -> CommandResult {
        let StringGreedyOrReply(prompt) = StringGreedyOrReply::convert(ctx, &arguments).await?.0;

        // Check if the http status code is valid
        // https://developer.mozilla.org/en-US/docs/Web/HTTP/Status
        let valid_codes = [
            100, 101, 102, 103, 200, 201, 202, 203, 204, 205, 206, 207, 208, 226, 300, 301, 302,
            303, 304, 305, 306, 307, 308, 400, 401, 402, 403, 404, 405, 406, 407, 408, 409, 410,
            411, 412, 413, 414, 415, 416, 417, 418, 421, 422, 423, 424, 425, 426, 428, 429, 431,
            451, 500, 501, 502, 503, 504, 505, 506, 507, 508, 510, 511,
        ];

        let status = match prompt.parse::<u16>() {
            Ok(status) if valid_codes.contains(&status) => status,
            _ => return Err(CommandError::Custom("error: invalid HTTP status code".to_string())),
        };

        let url = format!("https://http.cat/{}", status);
        ctx.reply_webpage(url).await?;
        Ok(())
    }
}
