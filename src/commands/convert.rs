//! inspired by <https://github.com/lunush/rates>

use std::borrow::Cow;
use std::time::{Duration, Instant};
use std::{fmt, iter};

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
                    if !amount.is_normal() {
                        return Err(ConversionError::BadArgument(Cow::Owned(format!(
                            "{amount} is not a valid amount."
                        ))));
                    }

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
        if self.0.fract() == 0. {
            return write!(f, "{}", self.0);
        }

        let significant_digits = -self.0.log10().floor();
        #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let precision = (significant_digits + 2.).max(0.) as usize;
        let number = format!("{:.*}", precision, self.0);

        write!(f, "{}", number.trim_end_matches('0').trim_end_matches('.'))
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

        let (source_currency, amount_eur) =
            match get_fiat_rate(&currencies.fiat, &arguments.currency) {
                Some((currency, rate)) => (currency, arguments.amount / rate),
                None => match get_crypto_price(&currencies.crypto, &arguments.currency) {
                    Some((currency, price)) => (currency, arguments.amount * price),
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
                        Some((symbol, price)) => Ok((symbol, amount_eur / price)),
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

        let joiner = if target_currencies.len() <= 2 { " = " } else { "\n= " };

        let response = target_currencies
            .into_iter()
            .map(|target_currency| {
                format!("{} {}", FormatAmount(target_currency.1), target_currency.0)
            })
            .collect::<Vec<_>>()
            .join(joiner);

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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_format_amount() {
        assert_eq!(format!("{}", FormatAmount(0.)), "0");
        assert_eq!(format!("{}", FormatAmount(1.)), "1");
        assert_eq!(format!("{}", FormatAmount(2.5)), "2.5");
        assert_eq!(format!("{}", FormatAmount(2.501)), "2.5");
        assert_eq!(format!("{}", FormatAmount(10.)), "10");
        assert_eq!(format!("{}", FormatAmount(25.)), "25");
        assert_eq!(format!("{}", FormatAmount(100.)), "100");
        assert_eq!(format!("{}", FormatAmount(1000.)), "1000");
        assert_eq!(format!("{}", FormatAmount(0.1)), "0.1");
        assert_eq!(format!("{}", FormatAmount(0.25)), "0.25");
        assert_eq!(format!("{}", FormatAmount(0.01)), "0.01");
        assert_eq!(format!("{}", FormatAmount(0.001)), "0.001");
        assert_eq!(format!("{}", FormatAmount(0.0001)), "0.0001");
        assert_eq!(format!("{}", FormatAmount(0.1234)), "0.123");
        assert_eq!(format!("{}", FormatAmount(0.01234)), "0.0123");
        assert_eq!(format!("{}", FormatAmount(0.001234)), "0.00123");
        assert_eq!(format!("{}", FormatAmount(0.0001234)), "0.000123");
    }
}
