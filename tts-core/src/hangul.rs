/// 한글 유니코드 자모 분해 및 음소 변환 모듈

use serde::{Deserialize, Serialize};

// 한글 유니코드 범위
const HANGUL_BASE: u32 = 0xAC00;
const HANGUL_END: u32 = 0xD7A3;

// 초성 19개
const CHOSEONG: [&str; 19] = [
    "ㄱ", "ㄲ", "ㄴ", "ㄷ", "ㄸ", "ㄹ", "ㅁ", "ㅂ", "ㅃ", "ㅅ",
    "ㅆ", "ㅇ", "ㅈ", "ㅉ", "ㅊ", "ㅋ", "ㅌ", "ㅍ", "ㅎ",
];

// 중성 21개
const JUNGSEONG: [&str; 21] = [
    "ㅏ", "ㅐ", "ㅑ", "ㅒ", "ㅓ", "ㅔ", "ㅕ", "ㅖ", "ㅗ", "ㅘ",
    "ㅙ", "ㅚ", "ㅛ", "ㅜ", "ㅝ", "ㅞ", "ㅟ", "ㅠ", "ㅡ", "ㅢ",
    "ㅣ",
];

// 종성 28개 (첫 번째는 종성 없음)
const JONGSEONG: [&str; 28] = [
    "", "ㄱ", "ㄲ", "ㄳ", "ㄴ", "ㄵ", "ㄶ", "ㄷ", "ㄹ", "ㄺ",
    "ㄻ", "ㄼ", "ㄽ", "ㄾ", "ㄿ", "ㅀ", "ㅁ", "ㅂ", "ㅄ", "ㅅ",
    "ㅆ", "ㅇ", "ㅈ", "ㅊ", "ㅋ", "ㅌ", "ㅍ", "ㅎ",
];

// 초성 → IPA 음소 매핑
fn choseong_to_phoneme(jamo: &str, is_word_initial: bool) -> &'static str {
    match jamo {
        "ㄱ" => if is_word_initial { "k" } else { "g" },
        "ㄲ" => "kk",
        "ㄴ" => "n",
        "ㄷ" => if is_word_initial { "t" } else { "d" },
        "ㄸ" => "tt",
        "ㄹ" => "r",
        "ㅁ" => "m",
        "ㅂ" => if is_word_initial { "p" } else { "b" },
        "ㅃ" => "pp",
        "ㅅ" => "s",
        "ㅆ" => "ss",
        "ㅇ" => "", // 초성 ㅇ은 무음
        "ㅈ" => if is_word_initial { "ch" } else { "j" },
        "ㅉ" => "jj",
        "ㅊ" => "ch",
        "ㅋ" => "kh",
        "ㅌ" => "th",
        "ㅍ" => "ph",
        "ㅎ" => "h",
        _ => "?",
    }
}

// 중성 → IPA 음소 매핑
fn jungseong_to_phoneme(jamo: &str) -> &'static str {
    match jamo {
        "ㅏ" => "a",
        "ㅐ" => "ae",
        "ㅑ" => "ya",
        "ㅒ" => "yae",
        "ㅓ" => "eo",
        "ㅔ" => "e",
        "ㅕ" => "yeo",
        "ㅖ" => "ye",
        "ㅗ" => "o",
        "ㅘ" => "wa",
        "ㅙ" => "wae",
        "ㅚ" => "oe",
        "ㅛ" => "yo",
        "ㅜ" => "u",
        "ㅝ" => "wo",
        "ㅞ" => "we",
        "ㅟ" => "wi",
        "ㅠ" => "yu",
        "ㅡ" => "eu",
        "ㅢ" => "ui",
        "ㅣ" => "i",
        _ => "?",
    }
}

// 종성 → IPA 음소 매핑
fn jongseong_to_phoneme(jamo: &str) -> &'static str {
    match jamo {
        "" => "",
        "ㄱ" | "ㄲ" | "ㅋ" => "k",
        "ㄳ" => "k",
        "ㄴ" => "n",
        "ㄵ" => "n",
        "ㄶ" => "n",
        "ㄷ" | "ㅅ" | "ㅆ" | "ㅈ" | "ㅊ" | "ㅌ" | "ㅎ" => "t",
        "ㄹ" => "l",
        "ㄺ" => "k",
        "ㄻ" => "m",
        "ㄼ" => "l",
        "ㄽ" => "l",
        "ㄾ" => "l",
        "ㄿ" => "l",
        "ㅀ" => "l",
        "ㅁ" => "m",
        "ㅂ" | "ㅍ" => "p",
        "ㅄ" => "p",
        "ㅇ" => "ng",
        _ => "?",
    }
}

/// 자모 분해 결과
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JamoResult {
    pub character: String,
    pub choseong: String,
    pub jungseong: String,
    pub jongseong: String,
}

/// 음소 결과
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phoneme {
    pub symbol: String,
    pub phoneme_type: PhonemeType,
    pub source_char: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PhonemeType {
    Consonant,
    Vowel,
    Silence,
    Pause,
}

/// 한글 문자인지 확인
pub fn is_hangul(c: char) -> bool {
    let code = c as u32;
    code >= HANGUL_BASE && code <= HANGUL_END
}

/// 한글 문자를 자모로 분해
pub fn decompose_syllable(c: char) -> Option<JamoResult> {
    if !is_hangul(c) {
        return None;
    }

    let code = c as u32 - HANGUL_BASE;
    let cho = (code / (21 * 28)) as usize;
    let jung = ((code % (21 * 28)) / 28) as usize;
    let jong = (code % 28) as usize;

    Some(JamoResult {
        character: c.to_string(),
        choseong: CHOSEONG[cho].to_string(),
        jungseong: JUNGSEONG[jung].to_string(),
        jongseong: JONGSEONG[jong].to_string(),
    })
}

/// 텍스트 전체를 자모로 분해
pub fn decompose_text(text: &str) -> Vec<JamoResult> {
    text.chars()
        .filter_map(|c| decompose_syllable(c))
        .collect()
}

/// 한국어 음운 규칙 적용 (자모 분해 결과를 변환)
/// 연음, 경음화, 비음화, 유음화, 구개음화
pub fn apply_phonological_rules(jamo_list: &mut Vec<JamoResult>) {
    let len = jamo_list.len();
    if len < 2 {
        return;
    }

    for i in 0..len - 1 {
        let jong = jamo_list[i].jongseong.clone();
        let next_cho = jamo_list[i + 1].choseong.clone();

        if jong.is_empty() {
            continue;
        }

        // 1. 연음 규칙: 종성 + 초성ㅇ → 종성이 다음 초성으로 이동
        //    + 구개음화 통합: ㄷ/ㅌ + 이 → ㅈ/ㅊ + 이
        if next_cho == "ㅇ" {
            if jamo_list[i + 1].jungseong == "ㅣ" {
                // 구개음화 (연음과 동시 적용)
                let palatalized = match jong.as_str() {
                    "ㄷ" => Some("ㅈ"),
                    "ㅌ" => Some("ㅊ"),
                    _ => None,
                };
                if let Some(p) = palatalized {
                    jamo_list[i].jongseong = String::new();
                    jamo_list[i + 1].choseong = p.to_string();
                    continue;
                }
            }
            let moved = jong.clone();
            jamo_list[i].jongseong = String::new();
            jamo_list[i + 1].choseong = moved;
            continue;
        }

        // 2. 비음화: 받침(ㄱ,ㄷ,ㅂ) + 비음(ㄴ,ㅁ) → 받침이 비음으로
        if matches!(next_cho.as_str(), "ㄴ" | "ㅁ") {
            let nasalized = match jong.as_str() {
                "ㄱ" | "ㄲ" | "ㅋ" | "ㄳ" | "ㄺ" => Some("ㅇ"),
                "ㄷ" | "ㅅ" | "ㅆ" | "ㅈ" | "ㅊ" | "ㅌ" | "ㅎ" => Some("ㄴ"),
                "ㅂ" | "ㅍ" | "ㄼ" | "ㅄ" => Some("ㅁ"),
                _ => None,
            };
            if let Some(n) = nasalized {
                jamo_list[i].jongseong = n.to_string();
                continue;
            }
        }

        // 3. 경음화: 받침(ㄱ,ㄷ,ㅂ) + 평음(ㄱ,ㄷ,ㅂ,ㅅ,ㅈ) → 평음이 경음으로
        if matches!(jong.as_str(), "ㄱ" | "ㄲ" | "ㅋ" | "ㄳ" | "ㄺ"
            | "ㄷ" | "ㅅ" | "ㅆ" | "ㅈ" | "ㅊ" | "ㅌ"
            | "ㅂ" | "ㅍ" | "ㄼ" | "ㅄ")
        {
            let tensified = match next_cho.as_str() {
                "ㄱ" => Some("ㄲ"),
                "ㄷ" => Some("ㄸ"),
                "ㅂ" => Some("ㅃ"),
                "ㅅ" => Some("ㅆ"),
                "ㅈ" => Some("ㅉ"),
                _ => None,
            };
            if let Some(t) = tensified {
                jamo_list[i + 1].choseong = t.to_string();
                continue;
            }
        }

        // 4. 유음화: ㄴ+ㄹ → ㄹ+ㄹ, ㄹ+ㄴ → ㄹ+ㄹ
        if jong == "ㄴ" && next_cho == "ㄹ" {
            jamo_list[i].jongseong = "ㄹ".to_string();
            continue;
        }
        if jong == "ㄹ" && next_cho == "ㄴ" {
            jamo_list[i + 1].choseong = "ㄹ".to_string();
            continue;
        }

        // 5. 구개음화: ㄷ/ㅌ + 이 → ㅈ/ㅊ + 이
        if jamo_list[i + 1].jungseong == "ㅣ" {
            let palatalized = match jong.as_str() {
                "ㄷ" => Some("ㅈ"),
                "ㅌ" => Some("ㅊ"),
                _ => None,
            };
            if let Some(p) = palatalized {
                jamo_list[i].jongseong = String::new();
                jamo_list[i + 1].choseong = p.to_string();
                continue;
            }
        }
    }
}

/// 자모 시퀀스를 음소로 변환 (음운 규칙 적용 후)
pub fn jamo_to_phonemes(jamo_list: &[JamoResult]) -> Vec<Phoneme> {
    let mut phonemes = Vec::new();

    for (i, jamo) in jamo_list.iter().enumerate() {
        let is_word_initial = i == 0
            || jamo_list.get(i.wrapping_sub(1)).map_or(true, |prev| {
                prev.character.chars().next().map_or(true, |c| c == ' ')
            });

        // 초성
        let cho_ph = choseong_to_phoneme(&jamo.choseong, is_word_initial);
        if !cho_ph.is_empty() {
            phonemes.push(Phoneme {
                symbol: cho_ph.to_string(),
                phoneme_type: PhonemeType::Consonant,
                source_char: jamo.character.clone(),
            });
        }

        // 중성
        let jung_ph = jungseong_to_phoneme(&jamo.jungseong);
        phonemes.push(Phoneme {
            symbol: jung_ph.to_string(),
            phoneme_type: PhonemeType::Vowel,
            source_char: jamo.character.clone(),
        });

        // 종성
        let jong_ph = jongseong_to_phoneme(&jamo.jongseong);
        if !jong_ph.is_empty() {
            phonemes.push(Phoneme {
                symbol: jong_ph.to_string(),
                phoneme_type: PhonemeType::Consonant,
                source_char: jamo.character.clone(),
            });
        }
    }

    phonemes
}

/// 텍스트 정규화 (숫자 → 한글, 기본 특수문자 처리)
pub fn normalize_text(text: &str) -> String {
    let mut result = String::new();

    for c in text.chars() {
        match c {
            '0' => result.push_str("영"),
            '1' => result.push_str("일"),
            '2' => result.push_str("이"),
            '3' => result.push_str("삼"),
            '4' => result.push_str("사"),
            '5' => result.push_str("오"),
            '6' => result.push_str("육"),
            '7' => result.push_str("칠"),
            '8' => result.push_str("팔"),
            '9' => result.push_str("구"),
            '.' | '!' | '?' => {
                result.push(c);
            }
            ',' => result.push(' '),
            _ if c.is_whitespace() || is_hangul(c) => result.push(c),
            _ => {} // 기타 문자 제거
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decompose_hangul() {
        let result = decompose_syllable('한').unwrap();
        assert_eq!(result.choseong, "ㅎ");
        assert_eq!(result.jungseong, "ㅏ");
        assert_eq!(result.jongseong, "ㄴ");
    }

    #[test]
    fn test_decompose_no_jongseong() {
        let result = decompose_syllable('하').unwrap();
        assert_eq!(result.choseong, "ㅎ");
        assert_eq!(result.jungseong, "ㅏ");
        assert_eq!(result.jongseong, "");
    }

    #[test]
    fn test_decompose_text() {
        let results = decompose_text("안녕");
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].choseong, "ㅇ");
        assert_eq!(results[0].jungseong, "ㅏ");
        assert_eq!(results[0].jongseong, "ㄴ");
        assert_eq!(results[1].choseong, "ㄴ");
        assert_eq!(results[1].jungseong, "ㅕ");
        assert_eq!(results[1].jongseong, "ㅇ");
    }

    #[test]
    fn test_phoneme_conversion() {
        let jamo = decompose_text("안녕하세요");
        let phonemes = jamo_to_phonemes(&jamo);
        let symbols: Vec<&str> = phonemes.iter().map(|p| p.symbol.as_str()).collect();
        // 안(ㅇ+ㅏ+ㄴ) 녕(ㄴ+ㅕ+ㅇ) 하(ㅎ+ㅏ) 세(ㅅ+ㅔ) 요(ㅇ+ㅛ)
        assert_eq!(symbols, vec!["a", "n", "n", "yeo", "ng", "h", "a", "s", "e", "yo"]);
    }

    #[test]
    fn test_normalize_numbers() {
        assert_eq!(normalize_text("123"), "일이삼");
    }

    #[test]
    fn test_non_hangul_ignored() {
        assert!(decompose_syllable('A').is_none());
        assert!(decompose_syllable('1').is_none());
    }

    #[test]
    fn test_liaison() {
        // 연음: "한인" → 하닌 (종성ㄴ + 초성ㅇ → 초성ㄴ)
        let mut jamo = decompose_text("한인");
        apply_phonological_rules(&mut jamo);
        assert_eq!(jamo[0].jongseong, "");    // 종성 비움
        assert_eq!(jamo[1].choseong, "ㄴ");   // 초성으로 이동
    }

    #[test]
    fn test_nasalization() {
        // 비음화: "국민" → 궁민 (ㄱ+ㅁ → ㅇ+ㅁ)
        let mut jamo = decompose_text("국민");
        apply_phonological_rules(&mut jamo);
        assert_eq!(jamo[0].jongseong, "ㅇ");
    }

    #[test]
    fn test_tensification() {
        // 경음화: "학교" → 학꾜 (ㄱ+ㄱ → ㄱ+ㄲ)
        let mut jamo = decompose_text("학교");
        apply_phonological_rules(&mut jamo);
        assert_eq!(jamo[1].choseong, "ㄲ");
    }

    #[test]
    fn test_lateralization() {
        // 유음화: "신라" → 실라 (ㄴ+ㄹ → ㄹ+ㄹ)
        let mut jamo = decompose_text("신라");
        apply_phonological_rules(&mut jamo);
        assert_eq!(jamo[0].jongseong, "ㄹ");
    }

    #[test]
    fn test_palatalization() {
        // 구개음화: "같이" → 가치 (ㅌ+이 → ㅊ+이)
        let mut jamo = decompose_text("같이");
        apply_phonological_rules(&mut jamo);
        assert_eq!(jamo[0].jongseong, "");
        assert_eq!(jamo[1].choseong, "ㅊ");
    }
}
