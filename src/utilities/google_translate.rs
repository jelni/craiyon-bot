use crate::commands::CommandError;

const LANGUAGES: [(&str, &str); 137] = [
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

pub fn language_supported(language_code: &str) -> bool {
    LANGUAGES.into_iter().any(|language| language.0 == language_code)
}

pub fn get_language_name(language_code: &str) -> Option<&str> {
    Some(LANGUAGES.into_iter().find(|language| language.0 == language_code)?.1)
}

fn parse_language(text: &str) -> Option<(&'static str, String)> {
    let text_lowercase = text.to_ascii_lowercase();
    let text_split = text_lowercase.split_ascii_whitespace().collect::<Vec<_>>();

    for (language_code, language) in LANGUAGES {
        for prefix in [&language.to_ascii_lowercase(), language_code] {
            if text_split.starts_with(
                &prefix.to_ascii_lowercase().split_ascii_whitespace().collect::<Vec<_>>(),
            ) {
                return Some((language_code, text[prefix.len()..].trim_start().to_owned()));
            }
        }
    }

    None
}

pub fn parse_command(text: String) -> (Option<&'static str>, Option<&'static str>, String) {
    let Some((first_language, text)) = parse_language(&text) else {
        return (None, None, text);
    };

    let Some((second_language, text)) = parse_language(&text) else {
        return (None, Some(first_language), text);
    };

    (Some(first_language), Some(second_language), text)
}

pub struct MissingTextToTranslate;

impl From<MissingTextToTranslate> for CommandError {
    fn from(_: MissingTextToTranslate) -> Self {
        CommandError::MissingArgument("text to translate")
    }
}
