//! inspired by <https://github.com/lunush/rates>

use std::borrow::Cow;
use std::fmt::{self};
use std::iter;
use std::time::{Duration, Instant};

use async_trait::async_trait;

use super::{CommandError, CommandResult, CommandTrait};
use crate::apis::coinranking::Coin;
use crate::apis::eurofxref::Rate;
use crate::apis::{coinranking, eurofxref};
use crate::utilities::bot_state::Currencies;
use crate::utilities::command_context::CommandContext;
use crate::utilities::convert_argument::{ConversionError, ConvertArgument};
use crate::utilities::message_entities::{self, ToEntity};
use crate::utilities::rate_limit::RateLimiter;

struct Arguments {
    amount: f64,
    currency: String,
    target_currencies: Vec<Cow<'static, str>>,
}

#[async_trait]
impl ConvertArgument for Arguments {
    async fn convert<'a>(
        ctx: &CommandContext,
        arguments: &'a str,
    ) -> Result<(Self, &'a str), ConversionError> {
        let (part, mut arguments) = Option::<String>::convert(ctx, arguments).await?;

        let (amount, currency) = match part {
            Some(part) => match part.parse::<f64>() {
                Ok(amount) => {
                    let (source_currency, rest) = String::convert(ctx, arguments).await?;
                    arguments = rest;
                    (amount, source_currency)
                }
                Err(_) => (1., part),
            },
            None => return Err(ConversionError::MissingArgument),
        };

        let mut target_currencies = Vec::new();

        loop {
            let (part, rest) = Option::<String>::convert(ctx, arguments).await?;
            arguments = rest;

            match part {
                Some(part) => {
                    if part.eq_ignore_ascii_case("to") {
                        let (currency, rest) = Option::<String>::convert(ctx, arguments).await?;
                        arguments = rest;

                        match currency {
                            Some(currency) => target_currencies.push(Cow::Owned(currency)),
                            None => break,
                        }
                    } else {
                        target_currencies.push(Cow::Owned(part));
                    }
                }
                None => break,
            }
        }

        if target_currencies.is_empty() {
            target_currencies.push(Cow::Borrowed("usd"));
        }

        Ok((Self { amount, currency, target_currencies }, arguments))
    }
}

struct FormatAmount(f64);

impl fmt::Display for FormatAmount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let significant_digits = -self.0.log10().floor();

        #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let precision = if significant_digits >= 0.
            && (self.0 * 10_f64.powi(significant_digits as i32)).fract() <= f64::EPSILON
        {
            significant_digits
        } else {
            (significant_digits + 2.).max(0.)
        } as usize;

        write!(f, "{:.*}", precision, self.0)
    }
}

pub struct Convert;

#[async_trait]
impl CommandTrait for Convert {
    fn command_names(&self) -> &[&str] {
        &["convert", "c", "rates", "rate", "price"]
    }

    fn description(&self) -> Option<&'static str> {
        Some("convert between popular currencies and cryptocurrencies")
    }

    fn rate_limit(&self) -> RateLimiter<i64> {
        RateLimiter::new(10, 30)
    }

    async fn execute(&self, ctx: &CommandContext, arguments: String) -> CommandResult {
        let arguments = Arguments::convert(ctx, &arguments).await?.0;

        let mut currencies = ctx.bot_state.currencies.lock().await;

        let currencies = match *currencies {
            Some(ref currencies) if currencies.updated_at.elapsed() < Duration::from_secs(3600) => {
                currencies
            }
            _ => {
                let fiat = eurofxref::daily(&ctx.bot_state.http_client).await?;
                let crypto = coinranking::coins(&ctx.bot_state.http_client).await?;
                *currencies = Some(Currencies { updated_at: Instant::now(), fiat, crypto });
                currencies.as_ref().unwrap()
            }
        };

        let (_, rate_usd) = get_fiat_rate(&currencies.fiat, "usd").unwrap();

        let (source_currency, amount_eur, amount_usd) =
            match get_fiat_rate(&currencies.fiat, &arguments.currency) {
                Some((currency, rate)) => {
                    let amount_eur = arguments.amount / rate;
                    (currency, amount_eur, amount_eur * rate_usd)
                }
                None => match get_crypto_price(&currencies.crypto, &arguments.currency) {
                    Some((currency, price)) => {
                        let amount_usd = arguments.amount * price;
                        (currency, amount_usd / rate_usd, amount_usd)
                    }
                    None => {
                        return Err(CommandError::CustomFormattedText(
                            message_entities::formatted_text(vec![
                                "could not find source currency ".text(),
                                arguments.currency.code(),
                            ]),
                        ));
                    }
                },
            };

        let target_currencies = iter::once(Ok((source_currency, arguments.amount)))
            .chain(arguments.target_currencies.into_iter().map(
                |target_currency| match get_fiat_rate(&currencies.fiat, &target_currency) {
                    Some((symbol, rate)) => Ok((symbol, amount_eur * rate)),
                    None => match get_crypto_price(&currencies.crypto, &target_currency) {
                        Some((symbol, price)) => Ok((symbol, amount_usd / price)),
                        None => Err(CommandError::CustomFormattedText(
                            message_entities::formatted_text(vec![
                                "could not find target currency ".text(),
                                target_currency.code(),
                            ]),
                        )),
                    },
                },
            ))
            .collect::<Result<Vec<_>, _>>()?;

        let response = target_currencies
            .into_iter()
            .map(|target_currency| {
                format!("{} {}", FormatAmount(target_currency.1), target_currency.0)
            })
            .collect::<Vec<_>>()
            .join(" = ");

        ctx.reply(response).await?;

        Ok(())
    }
}

fn get_fiat_rate<'a>(rates: &'a [Rate], currency: &str) -> Option<(&'a str, f64)> {
    if currency.eq_ignore_ascii_case("eur") {
        return Some(("EUR", 1.));
    }

    let coin = rates.iter().find(|rate| rate.currency.eq_ignore_ascii_case(currency))?;
    let rate = coin.rate.parse::<f64>().unwrap();

    Some((&coin.currency, rate))
}

fn get_crypto_price<'a>(coins: &'a [Coin], currency: &str) -> Option<(&'a str, f64)> {
    let coin = coins.iter().find(|rate| rate.symbol.eq_ignore_ascii_case(currency))?;
    let rate = coin.price.parse::<f64>().unwrap();

    Some((&coin.symbol, rate))
}
