// Text targets from the DeepL SDK and API documentation. Lexo resolves legacy
// EN/PT separately so every stored English or Portuguese target is explicit.
pub(crate) const TARGETS: &[(&str, &str)] = &[
    ("ACE", "Acehnese"),
    ("AF", "Afrikaans"),
    ("SQ", "Albanian"),
    ("AR", "Arabic"),
    ("AN", "Aragonese"),
    ("HY", "Armenian"),
    ("AS", "Assamese"),
    ("AY", "Aymara"),
    ("AZ", "Azerbaijani"),
    ("BA", "Bashkir"),
    ("EU", "Basque"),
    ("BE", "Belarusian"),
    ("BN", "Bengali"),
    ("BHO", "Bhojpuri"),
    ("BS", "Bosnian"),
    ("BR", "Breton"),
    ("BG", "Bulgarian"),
    ("MY", "Burmese"),
    ("YUE", "Cantonese"),
    ("CA", "Catalan"),
    ("CEB", "Cebuano"),
    ("ZH", "Chinese"),
    ("ZH-HANS", "Chinese (simplified)"),
    ("ZH-HANT", "Chinese (traditional)"),
    ("HR", "Croatian"),
    ("CS", "Czech"),
    ("DA", "Danish"),
    ("PRS", "Dari"),
    ("NL", "Dutch"),
    ("EN-US", "English (American)"),
    ("EN-GB", "English (British)"),
    ("EO", "Esperanto"),
    ("ET", "Estonian"),
    ("FI", "Finnish"),
    ("FR", "French"),
    ("FR-CA", "French (Canadian)"),
    ("FR-FR", "French (France)"),
    ("GL", "Galician"),
    ("KA", "Georgian"),
    ("DE", "German"),
    ("DE-DE", "German (Germany)"),
    ("DE-CH", "German (Switzerland)"),
    ("EL", "Greek"),
    ("GN", "Guarani"),
    ("GU", "Gujarati"),
    ("HT", "Haitian Creole"),
    ("HA", "Hausa"),
    ("HE", "Hebrew"),
    ("HI", "Hindi"),
    ("HU", "Hungarian"),
    ("IS", "Icelandic"),
    ("IG", "Igbo"),
    ("ID", "Indonesian"),
    ("GA", "Irish"),
    ("IT", "Italian"),
    ("JA", "Japanese"),
    ("JV", "Javanese"),
    ("PAM", "Kapampangan"),
    ("KK", "Kazakh"),
    ("GOM", "Konkani"),
    ("KO", "Korean"),
    ("KMR", "Kurdish (Kurmanji)"),
    ("CKB", "Kurdish (Sorani)"),
    ("KY", "Kyrgyz"),
    ("LA", "Latin"),
    ("LV", "Latvian"),
    ("LN", "Lingala"),
    ("LT", "Lithuanian"),
    ("LMO", "Lombard"),
    ("LB", "Luxembourgish"),
    ("MK", "Macedonian"),
    ("MAI", "Maithili"),
    ("MG", "Malagasy"),
    ("MS", "Malay"),
    ("ML", "Malayalam"),
    ("MT", "Maltese"),
    ("MI", "Maori"),
    ("MR", "Marathi"),
    ("MN", "Mongolian"),
    ("NE", "Nepali"),
    ("NB", "Norwegian (bokmål)"),
    ("OC", "Occitan"),
    ("OM", "Oromo"),
    ("PAG", "Pangasinan"),
    ("PS", "Pashto"),
    ("FA", "Persian"),
    ("PL", "Polish"),
    ("PT-BR", "Portuguese (Brazilian)"),
    ("PT-PT", "Portuguese (European)"),
    ("PA", "Punjabi"),
    ("QU", "Quechua"),
    ("RO", "Romanian"),
    ("RU", "Russian"),
    ("SA", "Sanskrit"),
    ("SR", "Serbian"),
    ("ST", "Sesotho"),
    ("SCN", "Sicilian"),
    ("SK", "Slovak"),
    ("SL", "Slovenian"),
    ("ES", "Spanish"),
    ("ES-419", "Spanish (Latin American)"),
    ("ES-ES", "Spanish (Spain)"),
    ("SU", "Sundanese"),
    ("SW", "Swahili"),
    ("SV", "Swedish"),
    ("TL", "Tagalog"),
    ("TG", "Tajik"),
    ("TA", "Tamil"),
    ("TT", "Tatar"),
    ("TE", "Telugu"),
    ("TH", "Thai"),
    ("TS", "Tsonga"),
    ("TN", "Tswana"),
    ("TR", "Turkish"),
    ("TK", "Turkmen"),
    ("UK", "Ukrainian"),
    ("UR", "Urdu"),
    ("UZ", "Uzbek"),
    ("VI", "Vietnamese"),
    ("CY", "Welsh"),
    ("WO", "Wolof"),
    ("XH", "Xhosa"),
    ("YI", "Yiddish"),
    ("ZU", "Zulu"),
];

pub(crate) fn is_valid_target(code: &str) -> bool {
    TARGETS.iter().any(|(target, _)| *target == code)
}

pub(crate) fn base(code: &str) -> &str {
    code.split('-').next().unwrap_or(code)
}

pub(crate) fn normalize_target(input: &str) -> Result<String, String> {
    let code = input.trim().to_ascii_uppercase();
    let target = match code.as_str() {
        "PT" => "PT-BR",
        "EN" => "EN-US",
        _ => &code,
    };
    if is_valid_target(target) {
        Ok(target.into())
    } else {
        Err("invalid_languages: Choose a supported DeepL target language.".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_case_and_legacy_codes() {
        assert_eq!(normalize_target("pt").unwrap(), "PT-BR");
        assert_eq!(normalize_target("EN").unwrap(), "EN-US");
        assert_eq!(normalize_target("es-419").unwrap(), "ES-419");
        assert_eq!(normalize_target("fr-ca").unwrap(), "FR-CA");
        assert_eq!(normalize_target("hi").unwrap(), "HI");
        assert!(normalize_target("klingon").is_err());
        assert!(normalize_target("").is_err());
    }

    #[test]
    fn reads_the_base_of_regional_and_unqualified_codes() {
        assert_eq!(base("PT-BR"), "PT");
        assert_eq!(base("ZH-HANS"), "ZH");
        assert_eq!(base("DE"), "DE");
        assert!(is_valid_target("DE"));
        assert!(!is_valid_target("EN"));
    }
}
