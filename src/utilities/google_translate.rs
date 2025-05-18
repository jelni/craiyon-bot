use std::borrow::Cow;

use async_trait::async_trait;

use super::command_context::CommandContext;
use super::convert_argument::{ConversionError, ConvertArgument};

pub const LANGUAGES: [(&str, &str); 194] = [
    ("ab", "Abkhaz"),
    ("ace", "Acehnese"),
    ("ach", "Acholi"),
    ("af", "Afrikaans"),
    ("sq", "Albanian"),
    ("alz", "Alur"),
    ("am", "Amharic"),
    ("ar", "Arabic"),
    ("hy", "Armenian"),
    ("as", "Assamese"),
    ("awa", "Awadhi"),
    ("ay", "Aymara"),
    ("az", "Azerbaijani"),
    ("ban", "Balinese"),
    ("bm", "Bambara"),
    ("ba", "Bashkir"),
    ("eu", "Basque"),
    ("btx", "Batak Karo"),
    ("bts", "Batak Simalungun"),
    ("bbc", "Batak Toba"),
    ("be", "Belarusian"),
    ("bem", "Bemba"),
    ("bn", "Bengali"),
    ("bew", "Betawi"),
    ("bho", "Bhojpuri"),
    ("bik", "Bikol"),
    ("bs", "Bosnian"),
    ("br", "Breton"),
    ("bg", "Bulgarian"),
    ("bua", "Buryat"),
    ("yue", "Cantonese"),
    ("ca", "Catalan"),
    ("ceb", "Cebuano"),
    ("ny", "Chichewa (Nyanja)"),
    ("zh", "Chinese (Simplified)"),
    ("zh-TW", "Chinese (Traditional)"),
    ("cv", "Chuvash"),
    ("co", "Corsican"),
    ("crh", "Crimean Tatar"),
    ("hr", "Croatian"),
    ("cs", "Czech"),
    ("da", "Danish"),
    ("din", "Dinka"),
    ("dv", "Divehi"),
    ("doi", "Dogri"),
    ("dov", "Dombe"),
    ("nl", "Dutch"),
    ("dz", "Dzongkha"),
    ("en", "English"),
    ("eo", "Esperanto"),
    ("et", "Estonian"),
    ("ee", "Ewe"),
    ("fj", "Fijian"),
    ("fil", "Filipino (Tagalog)"),
    ("fi", "Finnish"),
    ("fr", "French"),
    ("fr-FR", "French (French)"),
    ("fr-CA", "French (Canadian)"),
    ("fy", "Frisian"),
    ("ff", "Fulfulde"),
    ("gaa", "Ga"),
    ("gl", "Galician"),
    ("lg", "Ganda (Luganda)"),
    ("ka", "Georgian"),
    ("de", "German"),
    ("el", "Greek"),
    ("gn", "Guarani"),
    ("gu", "Gujarati"),
    ("ht", "Haitian Creole"),
    ("cnh", "Hakha Chin"),
    ("ha", "Hausa"),
    ("haw", "Hawaiian"),
    ("iw", "Hebrew"),
    ("hil", "Hiligaynon"),
    ("hi", "Hindi"),
    ("hmn", "Hmong"),
    ("hu", "Hungarian"),
    ("hrx", "Hunsrik"),
    ("is", "Icelandic"),
    ("ig", "Igbo"),
    ("ilo", "Iloko"),
    ("id", "Indonesian"),
    ("ga", "Irish"),
    ("it", "Italian"),
    ("ja", "Japanese"),
    ("jw", "Javanese"),
    ("kn", "Kannada"),
    ("pam", "Kapampangan"),
    ("kk", "Kazakh"),
    ("km", "Khmer"),
    ("cgg", "Kiga"),
    ("rw", "Kinyarwanda"),
    ("ktu", "Kituba"),
    ("gom", "Konkani"),
    ("ko", "Korean"),
    ("kri", "Krio"),
    ("ku", "Kurdish (Kurmanji)"),
    ("ckb", "Kurdish (Sorani)"),
    ("ky", "Kyrgyz"),
    ("lo", "Lao"),
    ("ltg", "Latgalian"),
    ("la", "Latin"),
    ("lv", "Latvian"),
    ("lij", "Ligurian"),
    ("li", "Limburgan"),
    ("ln", "Lingala"),
    ("lt", "Lithuanian"),
    ("lmo", "Lombard"),
    ("luo", "Luo"),
    ("lb", "Luxembourgish"),
    ("mk", "Macedonian"),
    ("mai", "Maithili"),
    ("mak", "Makassar"),
    ("mg", "Malagasy"),
    ("ms", "Malay"),
    ("ms-Arab", "Malay (Jawi)"),
    ("ml", "Malayalam"),
    ("mt", "Maltese"),
    ("mi", "Maori"),
    ("mr", "Marathi"),
    ("chm", "Meadow Mari"),
    ("mni-Mtei", "Meiteilon (Manipuri)"),
    ("min", "Minang"),
    ("lus", "Mizo"),
    ("mn", "Mongolian"),
    ("my", "Myanmar (Burmese)"),
    ("nr", "Ndebele (South)"),
    ("new", "Nepalbhasa (Newari)"),
    ("ne", "Nepali"),
    ("nso", "Northern Sotho (Sepedi)"),
    ("no", "Norwegian"),
    ("nus", "Nuer"),
    ("oc", "Occitan"),
    ("or", "Odia (Oriya)"),
    ("om", "Oromo"),
    ("pag", "Pangasinan"),
    ("pap", "Papiamento"),
    ("ps", "Pashto"),
    ("fa", "Persian"),
    ("pl", "Polish"),
    ("pt", "Portuguese"),
    ("pt-PT", "Portuguese (Portugal)"),
    ("pt-BR", "Portuguese (Brazil)"),
    ("pa", "Punjabi"),
    ("pa-Arab", "Punjabi (Shahmukhi)"),
    ("qu", "Quechua"),
    ("rom", "Romani"),
    ("ro", "Romanian"),
    ("rn", "Rundi"),
    ("ru", "Russian"),
    ("sm", "Samoan"),
    ("sg", "Sango"),
    ("sa", "Sanskrit"),
    ("gd", "Scots Gaelic"),
    ("sr", "Serbian"),
    ("st", "Sesotho"),
    ("crs", "Seychellois Creole"),
    ("shn", "Shan"),
    ("sn", "Shona"),
    ("scn", "Sicilian"),
    ("szl", "Silesian"),
    ("sd", "Sindhi"),
    ("si", "Sinhala (Sinhalese)"),
    ("sk", "Slovak"),
    ("sl", "Slovenian"),
    ("so", "Somali"),
    ("es", "Spanish"),
    ("su", "Sundanese"),
    ("sw", "Swahili"),
    ("ss", "Swati"),
    ("sv", "Swedish"),
    ("tg", "Tajik"),
    ("ta", "Tamil"),
    ("tt", "Tatar"),
    ("te", "Telugu"),
    ("tet", "Tetum"),
    ("th", "Thai"),
    ("ti", "Tigrinya"),
    ("ts", "Tsonga"),
    ("tn", "Tswana"),
    ("tr", "Turkish"),
    ("tk", "Turkmen"),
    ("ak", "Twi (Akan)"),
    ("uk", "Ukrainian"),
    ("ur", "Urdu"),
    ("ug", "Uyghur"),
    ("uz", "Uzbek"),
    ("vi", "Vietnamese"),
    ("cy", "Welsh"),
    ("xh", "Xhosa"),
    ("yi", "Yiddish"),
    ("yo", "Yoruba"),
    ("yua", "Yucatec Maya"),
    ("zu", "Zulu"),
];

pub fn get_language_name(language_code: &str) -> Option<&str> {
    Some(LANGUAGES.into_iter().find(|language| language.0 == language_code.to_ascii_lowercase())?.1)
}

#[derive(PartialEq, Eq)]
pub struct Language(pub &'static str);

#[async_trait]
impl ConvertArgument for Language {
    async fn convert<'a>(
        _: &CommandContext,
        arguments: &'a str,
    ) -> Result<(Self, &'a str), ConversionError> {
        let arguments = arguments.trim_ascii_start();

        if arguments.is_empty() {
            Err(ConversionError::MissingArgument)?;
        }

        let lowercase = arguments.to_ascii_lowercase();

        for (language_code, language) in LANGUAGES {
            for prefix in [language_code, &language.to_ascii_lowercase()] {
                if lowercase.starts_with(prefix) {
                    let rest = &arguments[prefix.len()..];
                    if rest.chars().next().is_none_or(|char| char.is_ascii_whitespace()) {
                        return Ok((Self(language_code), rest));
                    }
                }
            }
        }

        Err(ConversionError::BadArgument(Cow::Borrowed("unknown language code or name.")))
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

            return Ok((Self(None, target_language), arguments));
        };

        let Some((Language(second_language), rest)) = Language::convert(ctx, rest).await.ok()
        else {
            return Ok((Self(None, Cow::Borrowed(first_language)), rest));
        };

        Ok((Self(Some(first_language), Cow::Borrowed(second_language)), rest))
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
        assert!(matches!(result, Err(ConversionError::MissingArgument)));

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
