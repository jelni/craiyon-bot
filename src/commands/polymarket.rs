use async_trait::async_trait;
use time::macros;

use super::{CommandError, CommandResult, CommandTrait};
use crate::apis::polymarket;
use crate::utilities::command_context::CommandContext;
use crate::utilities::convert_argument::{ConvertArgument, StringGreedyOrReply};
use crate::utilities::message_entities::{self, ToEntity, ToEntityOwned, ToNestedEntity};

pub struct Polymarket;

#[async_trait]
impl CommandTrait for Polymarket {
    fn command_names(&self) -> &[&str] {
        &["polymarket", "poly"]
    }

    fn description(&self) -> Option<&'static str> {
        Some("check Polymarket bets on world events")
    }

    async fn execute(&self, ctx: &CommandContext, arguments: String) -> CommandResult {
        let StringGreedyOrReply(query) = ConvertArgument::convert(ctx, &arguments).await?.0;

        ctx.send_typing().await?;

        let events = polymarket::search_events(&ctx.bot_state.http_client, &query).await?;

        let Some(mut event) = events.into_iter().next() else {
            return Err(CommandError::Custom("no results found.".into()));
        };

        event.markets.retain(|market| {
            market.outcome_prices != ["0", "1"] && market.outcome_prices != ["0.0005", "0.9995"]
        });

        if event.markets.len() > 1 {
            event.markets.sort_by_cached_key(|market| {
                market.group_item_threshold.as_ref().unwrap().parse::<u32>().unwrap()
            });
        }

        let mut entities = vec![
            event.title.bold().text_url(format!("https://polymarket.com/event/{}", event.slug)),
            " (".text(),
            event
                .end_date
                .format(macros::format_description!("[year]-[month]-[day]"))
                .unwrap()
                .text_owned(),
            ")\n".text(),
        ];

        for (i, market) in event.markets.into_iter().enumerate() {
            if i > 0 {
                entities.push("\n".text());
            }

            if let Some(title) = market.group_item_title
                && !title.is_empty()
            {
                entities.extend([title.text_owned(), ": ".text()]);
            }

            let prices = market
                .outcome_prices
                .into_iter()
                .map(|price| format!("{:.1}%", price.parse::<f64>().unwrap() * 100.));

            if market.outcomes == ["Yes", "No"] {
                let price = prices.into_iter().next().unwrap();
                entities.push(price.text_owned());
            } else {
                for (i, (outcome, price)) in market.outcomes.into_iter().zip(prices).enumerate() {
                    if i > 0 {
                        entities.push(" / ".text());
                    }

                    entities.extend([outcome.text_owned(), " ".text(), price.text_owned()]);
                }
            }
        }

        ctx.reply_formatted_text(message_entities::formatted_text(entities)).await?;

        Ok(())
    }
}
