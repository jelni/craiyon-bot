use std::borrow::Cow;

use async_trait::async_trait;

use super::command_context::CommandContext;
use super::convert_argument::{ConversionError, ConvertArgument};

pub const LANGUAGES: [(&str, &str); 137] = [
    ("af", "Afrikaans"),
    ("sq", "Albanian"),
    ("am", "Amharic"),
    ("ar", "Arabic"),
    ("hy", "Armenian"),
    ("as", "Assamese"),
    ("ay", "Aymara"),
    ("az", "Azerbaijani"),
    ("bm", "Bambara"),
    ("eu", "Basque"),
    ("be", "Belarusian"),
    ("bn", "Bengali"),
    ("bho", "Bhojpuri"),
    ("bs", "Bosnian"),
    ("bg", "Bulgarian"),
    ("ca", "Catalan"),
    ("ceb", "Cebuano"),
    ("zh-cn", "Chinese (Simplified)"),
    ("zh", "Chinese (Simplified)"),
    ("zh-tw", "Chinese (Traditional)"),
    ("co", "Corsican"),
    ("hr", "Croatian"),
    ("cs", "Czech"),
    ("da", "Danish"),
    ("dv", "Dhivehi"),
    ("doi", "Dogri"),
    ("nl", "Dutch"),
    ("en", "English"),
    ("eo", "Esperanto"),
    ("et", "Estonian"),
    ("ee", "Ewe"),
    ("fil", "Filipino"),
    ("fi", "Finnish"),
    ("fr", "French"),
    ("fy", "Frisian"),
    ("gl", "Galician"),
    ("ka", "Georgian"),
    ("de", "German"),
    ("el", "Greek"),
    ("gn", "Guarani"),
    ("gu", "Gujarati"),
    ("ht", "Haitian Creole"),
    ("ha", "Hausa"),
    ("haw", "Hawaiian"),
    ("he", "Hebrew"),
    ("iw", "Hebrew"),
    ("hi", "Hindi"),
    ("hmn", "Hmong"),
    ("hu", "Hungarian"),
    ("is", "Icelandic"),
    ("ig", "Igbo"),
    ("ilo", "Ilocano"),
    ("id", "Indonesian"),
    ("ga", "Irish"),
    ("it", "Italian"),
    ("ja", "Japanese"),
    ("jv", "Javanese"),
    ("jw", "Javanese"),
    ("kn", "Kannada"),
    ("kk", "Kazakh"),
    ("km", "Khmer"),
    ("rw", "Kinyarwanda"),
    ("gom", "Konkani"),
    ("ko", "Korean"),
    ("kri", "Krio"),
    ("ku", "Kurdish"),
    ("ckb", "Kurdish"),
    ("ky", "Kyrgyz"),
    ("lo", "Lao"),
    ("la", "Latin"),
    ("lv", "Latvian"),
    ("ln", "Lingala"),
    ("lt", "Lithuanian"),
    ("lg", "Luganda"),
    ("lb", "Luxembourgish"),
    ("mk", "Macedonian"),
    ("mai", "Maithili"),
    ("mg", "Malagasy"),
    ("ms", "Malay"),
    ("ml", "Malayalam"),
    ("mt", "Maltese"),
    ("mi", "Maori"),
    ("mr", "Marathi"),
    ("mni-mtei", "Meiteilon"),
    ("lus", "Mizo"),
    ("mn", "Mongolian"),
    ("my", "Myanmar"),
    ("ne", "Nepali"),
    ("no", "Norwegian"),
    ("ny", "Nyanja"),
    ("or", "Odia"),
    ("om", "Oromo"),
    ("ps", "Pashto"),
    ("fa", "Persian"),
    ("pl", "Polish"),
    ("pt", "Portuguese"),
    ("pa", "Punjabi"),
    ("qu", "Quechua"),
    ("ro", "Romanian"),
    ("ru", "Russian"),
    ("sm", "Samoan"),
    ("sa", "Sanskrit"),
    ("gd", "Scots Gaelic"),
    ("nso", "Sepedi"),
    ("sr", "Serbian"),
    ("st", "Sesotho"),
    ("sn", "Shona"),
    ("sd", "Sindhi"),
    ("si", "Sinhala"),
    ("sk", "Slovak"),
    ("sl", "Slovenian"),
    ("so", "Somali"),
    ("es", "Spanish"),
    ("su", "Sundanese"),
    ("sw", "Swahili"),
    ("sv", "Swedish"),
    ("tl", "Tagalog"),
    ("tg", "Tajik"),
    ("ta", "Tamil"),
    ("tt", "Tatar"),
    ("te", "Telugu"),
    ("th", "Thai"),
    ("ti", "Tigrinya"),
    ("ts", "Tsonga"),
    ("tr", "Turkish"),
    ("tk", "Turkmen"),
    ("ak", "Twi"),
    ("uk", "Ukrainian"),
    ("ur", "Urdu"),
    ("ug", "Uyghur"),
    ("uz", "Uzbek"),
    ("vi", "Vietnamese"),
    ("cy", "Welsh"),
    ("xh", "Xhosa"),
    ("yi", "Yiddish"),
    ("yo", "Yoruba"),
    ("zu", "Zulu"),
];

pub fn get_language_name(language_code: &str) -> Option<&str> {
    Some(LANGUAGES.into_iter().find(|language| language.0 == language_code.to_ascii_lowercase())?.1)
}

#[derive(Debug, PartialEq, Eq)]
pub struct Language(pub &'static str);

#[async_trait]
impl ConvertArgument for Language {
    async fn convert<'a>(
        _: &CommandContext,
        arguments: &'a str,
    ) -> Result<(Self, &'a str), ConversionError> {
        let arguments = arguments.trim_start();

        if arguments.is_empty() {
            Err(ConversionError::MissingArgument)?;
        }

        let lowercase = arguments.to_ascii_lowercase();

        for (language_code, language) in LANGUAGES {
            for prefix in [language_code, &language.to_ascii_lowercase()] {
                if lowercase.starts_with(prefix) {
                    let rest = &arguments[prefix.len()..];
                    if rest.chars().next().map_or(true, |char| char.is_ascii_whitespace()) {
                        return Ok((Self(language_code), rest));
                    }
                }
            }
        }

        Err(ConversionError::BadArgument("unknown language code or name."))
    }
}

pub struct SourceTargetLanguages(pub Option<&'static str>, pub Cow<'static, str>);

#[async_trait]
impl ConvertArgument for SourceTargetLanguages {
    async fn convert<'a>(
        ctx: &CommandContext,
        arguments: &'a str,
    ) -> Result<(Self, &'a str), ConversionError> {
        let Some((Language(first_language), rest)) = Language::convert(ctx, arguments).await.ok()
        else {
            let target_language = if ctx.user.language_code.is_empty() {
                Cow::Borrowed("en")
            } else {
                Cow::Owned(ctx.user.language_code.clone())
            };

            return Ok((SourceTargetLanguages(None, target_language), arguments));
        };

        let Some((Language(second_language), rest)) = Language::convert(ctx, rest).await.ok()
        else {
            return Ok((SourceTargetLanguages(None, Cow::Borrowed(first_language)), rest));
        };

        Ok((SourceTargetLanguages(Some(first_language), Cow::Borrowed(second_language)), rest))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::utilities::test_fixtures;

    #[tokio::test]
    async fn test_language_converter() {
        let ctx = test_fixtures::command_context();

        let result = Language::convert(&ctx, "").await;
        assert_eq!(result, Err(ConversionError::MissingArgument));

        let result = <Language>::convert(&ctx, "foo").await;
        let Err(ConversionError::BadArgument(_)) = result else {
            panic!("expected BadArgument error");
        };

        let (Language(argument), rest) = ConvertArgument::convert(&ctx, "en").await.unwrap();
        assert_eq!(argument, "en");
        assert_eq!(rest, "");

        let (Language(argument), rest) = ConvertArgument::convert(&ctx, "en foo").await.unwrap();
        assert_eq!(argument, "en");
        assert_eq!(rest, " foo");

        let (Language(argument), rest) = ConvertArgument::convert(&ctx, "english").await.unwrap();
        assert_eq!(argument, "en");
        assert_eq!(rest, "");

        let (Language(argument), rest) =
            ConvertArgument::convert(&ctx, "english FOO").await.unwrap();
        assert_eq!(argument, "en");
        assert_eq!(rest, " FOO");

        let (Language(argument), rest) =
            ConvertArgument::convert(&ctx, "ENGLISH foo").await.unwrap();
        assert_eq!(argument, "en");
        assert_eq!(rest, " foo");

        let (Language(argument), rest) =
            ConvertArgument::convert(&ctx, "chinese (simplified)").await.unwrap();
        assert_eq!(argument, "zh-cn");
        assert_eq!(rest, "");

        let result = <Language>::convert(&ctx, "chinese").await;
        let Err(ConversionError::BadArgument(_)) = result else {
            panic!("expected BadArgument error");
        };

        let result = <Language>::convert(&ctx, "chinese  (simplified)").await;
        let Err(ConversionError::BadArgument(_)) = result else {
            panic!("expected BadArgument error");
        };

        let (Language(argument), rest) =
            ConvertArgument::convert(&ctx, "chinese (simplified) FOO").await.unwrap();
        assert_eq!(argument, "zh-cn");
        assert_eq!(rest, " FOO");

        let (Language(argument), rest) =
            ConvertArgument::convert(&ctx, "CHINESE (SIMPLIFIED) foo").await.unwrap();
        assert_eq!(argument, "zh-cn");
        assert_eq!(rest, " foo");
    }

    #[tokio::test]
    async fn test_source_target_languages_converter() {
        let ctx = test_fixtures::command_context();

        let (SourceTargetLanguages(source_language, target_language), rest) =
            ConvertArgument::convert(&ctx, "").await.unwrap();
        assert_eq!(source_language, None);
        assert_eq!(target_language, "user_language_code");
        assert_eq!(rest, "");

        let (SourceTargetLanguages(source_language, target_language), rest) =
            ConvertArgument::convert(&ctx, "en").await.unwrap();
        assert_eq!(source_language, None);
        assert_eq!(target_language, "en");
        assert_eq!(rest, "");

        let (SourceTargetLanguages(source_language, target_language), rest) =
            ConvertArgument::convert(&ctx, "en foo").await.unwrap();
        assert_eq!(source_language, None);
        assert_eq!(target_language, "en");
        assert_eq!(rest, " foo");

        let (SourceTargetLanguages(source_language, target_language), rest) =
            ConvertArgument::convert(&ctx, "chinese (simplified) english").await.unwrap();
        assert_eq!(source_language, Some("zh-cn"));
        assert_eq!(target_language, "en");
        assert_eq!(rest, "");

        let (SourceTargetLanguages(source_language, target_language), rest) =
            ConvertArgument::convert(&ctx, "chinese (simplified) english foo").await.unwrap();
        assert_eq!(source_language, Some("zh-cn"));
        assert_eq!(target_language, "en");
        assert_eq!(rest, " foo");
    }
}
