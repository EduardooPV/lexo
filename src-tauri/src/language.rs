const PT_WORDS: &[&str] = &[
    " o ",
    " a ",
    " os ",
    " as ",
    " um ",
    " uma ",
    " de ",
    " do ",
    " da ",
    " dos ",
    " das ",
    " em ",
    " no ",
    " na ",
    " que ",
    " e ",
    " ou ",
    " mas ",
    " com ",
    " sem ",
    " por ",
    " para ",
    " se ",
    " eu ",
    " voce ",
    " ele ",
    " ela ",
    " nos ",
    " eles ",
    " meu ",
    " minha ",
    " seu ",
    " sua ",
    " isso ",
    " isto ",
    " aqui ",
    " ali ",
    " nao ",
    " sim ",
    " muito ",
    " mais ",
    " menos ",
    " tudo ",
    " nada ",
    " bom ",
    " boa ",
    " dia ",
    " noite ",
    " obrigado ",
    " obrigada ",
    " ola ",
    " porque ",
    " quando ",
    " onde ",
    " como ",
    " quem ",
    " qual ",
    " ser ",
    " estar ",
    " ter ",
    " fazer ",
    " vou ",
    " vai ",
    " esta ",
    " sao ",
    " foi ",
    " tem ",
    " quero ",
    " preciso ",
    " gosto ",
    " casa ",
    " agua ",
    " hoje ",
    " amanha ",
];

const EN_WORDS: &[&str] = &[
    " the ",
    " an ",
    " of ",
    " to ",
    " in ",
    " on ",
    " at ",
    " is ",
    " are ",
    " was ",
    " were ",
    " be ",
    " been ",
    " and ",
    " or ",
    " but ",
    " with ",
    " without ",
    " for ",
    " if ",
    " i ",
    " you ",
    " he ",
    " she ",
    " we ",
    " they ",
    " it ",
    " my ",
    " your ",
    " his ",
    " her ",
    " this ",
    " that ",
    " these ",
    " those ",
    " here ",
    " there ",
    " no ",
    " yes ",
    " not ",
    " very ",
    " more ",
    " less ",
    " all ",
    " nothing ",
    " good ",
    " day ",
    " night ",
    " thanks ",
    " thank ",
    " hello ",
    " hi ",
    " because ",
    " when ",
    " where ",
    " how ",
    " who ",
    " which ",
    " do ",
    " does ",
    " did ",
    " have ",
    " has ",
    " had ",
    " will ",
    " would ",
    " can ",
    " want ",
    " need ",
    " like ",
    " house ",
    " water ",
    " today ",
    " tomorrow ",
    " go ",
    " going ",
    " me ",
    " please ",
];

pub(crate) fn guess_target(text: &str) -> &'static str {
    let lowered = text.to_lowercase();

    if lowered.chars().any(|c| "ãõáàâêôçéíóúü".contains(c)) {
        return "EN";
    }

    let letters: String = lowered
        .chars()
        .map(|c| {
            if c.is_ascii_alphabetic() || c == '\'' {
                c
            } else {
                ' '
            }
        })
        .collect();
    let padded = format!(
        " {} ",
        letters.split_whitespace().collect::<Vec<_>>().join(" ")
    );

    let mut pt = PT_WORDS.iter().filter(|w| padded.contains(**w)).count() as i32;
    let mut en = EN_WORDS.iter().filter(|w| padded.contains(**w)).count() as i32;

    for pattern in [
        "ção", "ções", "lh", "nh", "mente ", "ando ", "endo ", "inho ", "inha ",
    ] {
        pt += lowered.matches(pattern).count() as i32;
    }
    for pattern in ["th", "wh", "ght", "ing ", "tion ", "ly "] {
        en += lowered.matches(pattern).count() as i32;
    }

    if pt != en {
        return if pt > en { "EN" } else { "PT" };
    }
    if lowered.contains(['w', 'y', 'k']) {
        "PT"
    } else {
        "EN"
    }
}

pub(crate) fn normalize_target(target: &str) -> String {
    if target.to_uppercase().starts_with("PT") {
        "PT".to_string()
    } else {
        "EN".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod guess_target {
        use super::*;

        #[test]
        fn portuguese_diacritics_are_decisive_on_their_own() {
            assert_eq!(guess_target("ação"), "EN");
            assert_eq!(guess_target("você"), "EN");
            assert_eq!(guess_target("não"), "EN");
        }

        #[test]
        fn reads_plain_portuguese_without_accents() {
            assert_eq!(guess_target("preciso revisar isso antes do deploy"), "EN");
            assert_eq!(guess_target("bom dia para voce"), "EN");
        }

        #[test]
        fn reads_plain_english() {
            assert_eq!(
                guess_target("I need to review this before the deploy"),
                "PT"
            );
            assert_eq!(guess_target("the build failed on the runner"), "PT");
        }

        #[test]
        fn never_panics_on_degenerate_input() {
            for text in ["", " ", "123", "!@#$", "\n\t"] {
                let _ = guess_target(text);
            }
        }
    }

    mod normalize_target {
        use super::*;

        #[test]
        fn accepts_either_case_and_regional_variants() {
            assert_eq!(normalize_target("PT"), "PT");
            assert_eq!(normalize_target("pt"), "PT");
            assert_eq!(normalize_target("pt-BR"), "PT");
            assert_eq!(normalize_target("EN"), "EN");
            assert_eq!(normalize_target("en-US"), "EN");
        }

        #[test]
        fn anything_unrecognised_falls_back_to_english() {
            assert_eq!(normalize_target("klingon"), "EN");
            assert_eq!(normalize_target(""), "EN");
        }
    }
}
