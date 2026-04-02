/// 영어 텍스트 분석 및 G2P(Grapheme-to-Phoneme) 변환 모듈

use serde::{Deserialize, Serialize};
use crate::hangul::PhonemeType;

/// 영어 음소 결과
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnglishPhoneme {
    pub symbol: String,
    pub phoneme_type: PhonemeType,
    pub source: String,
}

/// 영어 텍스트 정규화
pub fn normalize_english(text: &str) -> String {
    let mut result = String::new();
    for c in text.chars() {
        match c {
            '0' => result.push_str("zero"),
            '1' => result.push_str("one"),
            '2' => result.push_str("two"),
            '3' => result.push_str("three"),
            '4' => result.push_str("four"),
            '5' => result.push_str("five"),
            '6' => result.push_str("six"),
            '7' => result.push_str("seven"),
            '8' => result.push_str("eight"),
            '9' => result.push_str("nine"),
            '.' | '!' | '?' | '\'' => result.push(c),
            ',' => result.push(' '),
            _ if c.is_ascii_alphabetic() || c.is_whitespace() => result.push(c.to_ascii_lowercase()),
            _ => {}
        }
    }
    result
}

/// 규칙 기반 영어 G2P 변환
/// 간단한 규칙 기반 접근: 일반적인 영어 단어 패턴 매핑
pub fn english_g2p(text: &str) -> Vec<EnglishPhoneme> {
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut phonemes = Vec::new();

    for word in words.iter() {
        // pause는 insert_pauses()가 처리 — 여기선 삽입 안 함
        let word_phonemes = word_to_phonemes(word);
        phonemes.extend(word_phonemes);
    }

    phonemes
}

/// 단어를 음소로 변환 (규칙 기반)
fn word_to_phonemes(word: &str) -> Vec<EnglishPhoneme> {
    // 일반적인 단어 사전
    if let Some(phonemes) = lookup_dictionary(word) {
        return phonemes;
    }

    // 규칙 기반 변환
    let mut result = Vec::new();
    let chars: Vec<char> = word.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let remaining = &word[i..];

        // 다중 문자 패턴 매칭 (긴 것 먼저)
        if let Some((phoneme, ptype, consumed)) = match_pattern(remaining, i, len) {
            result.push(EnglishPhoneme {
                symbol: phoneme.to_string(),
                phoneme_type: ptype,
                source: word[i..i + consumed].to_string(),
            });
            i += consumed;
        } else {
            // 단일 문자
            if let Some((ph, pt)) = single_char_phoneme(chars[i]) {
                result.push(EnglishPhoneme {
                    symbol: ph.to_string(),
                    phoneme_type: pt,
                    source: chars[i].to_string(),
                });
            }
            i += 1;
        }
    }

    result
}

fn match_pattern(remaining: &str, pos: usize, word_len: usize) -> Option<(&'static str, PhonemeType, usize)> {
    // 3문자 패턴
    if remaining.len() >= 3 {
        match &remaining[..3] {
            "tch" => return Some(("ch", PhonemeType::Consonant, 3)),
            "igh" => return Some(("ai", PhonemeType::Vowel, 3)),
            "tion" if remaining.len() >= 4 => return Some(("sh-n", PhonemeType::Consonant, 4)),
            "ous" => return Some(("ah-s", PhonemeType::Vowel, 3)),
            "ing" => return Some(("ih-ng", PhonemeType::Consonant, 3)),
            _ => {}
        }
    }

    // 2문자 패턴
    if remaining.len() >= 2 {
        match &remaining[..2] {
            "th" => return Some(("th", PhonemeType::Consonant, 2)),
            "sh" => return Some(("sh", PhonemeType::Consonant, 2)),
            "ch" => return Some(("ch", PhonemeType::Consonant, 2)),
            "ph" => return Some(("f", PhonemeType::Consonant, 2)),
            "wh" => return Some(("w", PhonemeType::Consonant, 2)),
            "ck" => return Some(("k", PhonemeType::Consonant, 2)),
            "ng" => return Some(("ng", PhonemeType::Consonant, 2)),
            "qu" => return Some(("kw", PhonemeType::Consonant, 2)),
            "ee" => return Some(("iy", PhonemeType::Vowel, 2)),
            "ea" => return Some(("iy", PhonemeType::Vowel, 2)),
            "oo" => return Some(("uw", PhonemeType::Vowel, 2)),
            "ou" => return Some(("aw", PhonemeType::Vowel, 2)),
            "ow" => {
                if pos + 2 == word_len { return Some(("ow", PhonemeType::Vowel, 2)); }
                return Some(("aw", PhonemeType::Vowel, 2));
            }
            "ai" | "ay" => return Some(("ey", PhonemeType::Vowel, 2)),
            "oi" | "oy" => return Some(("oy", PhonemeType::Vowel, 2)),
            "ar" => return Some(("aa-r", PhonemeType::Vowel, 2)),
            "er" | "ir" | "ur" => return Some(("er", PhonemeType::Vowel, 2)),
            "or" => return Some(("ao-r", PhonemeType::Vowel, 2)),
            "al" => return Some(("ao-l", PhonemeType::Vowel, 2)),
            _ => {}
        }
    }

    None
}

fn single_char_phoneme(c: char) -> Option<(&'static str, PhonemeType)> {
    match c {
        'a' => Some(("ae", PhonemeType::Vowel)),
        'e' => Some(("eh", PhonemeType::Vowel)),
        'i' => Some(("ih", PhonemeType::Vowel)),
        'o' => Some(("aa", PhonemeType::Vowel)),
        'u' => Some(("ah", PhonemeType::Vowel)),
        'b' => Some(("b", PhonemeType::Consonant)),
        'c' => Some(("k", PhonemeType::Consonant)),
        'd' => Some(("d", PhonemeType::Consonant)),
        'f' => Some(("f", PhonemeType::Consonant)),
        'g' => Some(("g", PhonemeType::Consonant)),
        'h' => Some(("hh", PhonemeType::Consonant)),
        'j' => Some(("jh", PhonemeType::Consonant)),
        'k' => Some(("k", PhonemeType::Consonant)),
        'l' => Some(("l", PhonemeType::Consonant)),
        'm' => Some(("m", PhonemeType::Consonant)),
        'n' => Some(("n", PhonemeType::Consonant)),
        'p' => Some(("p", PhonemeType::Consonant)),
        'r' => Some(("r", PhonemeType::Consonant)),
        's' => Some(("s", PhonemeType::Consonant)),
        't' => Some(("t", PhonemeType::Consonant)),
        'v' => Some(("v", PhonemeType::Consonant)),
        'w' => Some(("w", PhonemeType::Consonant)),
        'x' => Some(("k-s", PhonemeType::Consonant)),
        'y' => Some(("y", PhonemeType::Consonant)),
        'z' => Some(("z", PhonemeType::Consonant)),
        _ => None,
    }
}

/// 자주 사용되는 단어 사전
fn lookup_dictionary(word: &str) -> Option<Vec<EnglishPhoneme>> {
    let phonemes: Vec<(&str, PhonemeType)> = match word {
        "hello" => vec![("hh", PhonemeType::Consonant), ("eh", PhonemeType::Vowel), ("l", PhonemeType::Consonant), ("ow", PhonemeType::Vowel)],
        "world" => vec![("w", PhonemeType::Consonant), ("er", PhonemeType::Vowel), ("l", PhonemeType::Consonant), ("d", PhonemeType::Consonant)],
        "the" => vec![("th", PhonemeType::Consonant), ("ah", PhonemeType::Vowel)],
        "a" => vec![("ah", PhonemeType::Vowel)],
        "is" => vec![("ih", PhonemeType::Vowel), ("z", PhonemeType::Consonant)],
        "are" => vec![("aa", PhonemeType::Vowel), ("r", PhonemeType::Consonant)],
        "to" => vec![("t", PhonemeType::Consonant), ("uw", PhonemeType::Vowel)],
        "you" => vec![("y", PhonemeType::Consonant), ("uw", PhonemeType::Vowel)],
        "and" => vec![("ae", PhonemeType::Vowel), ("n", PhonemeType::Consonant), ("d", PhonemeType::Consonant)],
        "in" => vec![("ih", PhonemeType::Vowel), ("n", PhonemeType::Consonant)],
        "it" => vec![("ih", PhonemeType::Vowel), ("t", PhonemeType::Consonant)],
        "that" => vec![("th", PhonemeType::Consonant), ("ae", PhonemeType::Vowel), ("t", PhonemeType::Consonant)],
        "this" => vec![("th", PhonemeType::Consonant), ("ih", PhonemeType::Vowel), ("s", PhonemeType::Consonant)],
        "good" => vec![("g", PhonemeType::Consonant), ("uh", PhonemeType::Vowel), ("d", PhonemeType::Consonant)],
        "my" => vec![("m", PhonemeType::Consonant), ("ai", PhonemeType::Vowel)],
        "name" => vec![("n", PhonemeType::Consonant), ("ey", PhonemeType::Vowel), ("m", PhonemeType::Consonant)],
        "yes" => vec![("y", PhonemeType::Consonant), ("eh", PhonemeType::Vowel), ("s", PhonemeType::Consonant)],
        "no" => vec![("n", PhonemeType::Consonant), ("ow", PhonemeType::Vowel)],
        "how" => vec![("hh", PhonemeType::Consonant), ("aw", PhonemeType::Vowel)],
        "what" => vec![("w", PhonemeType::Consonant), ("ah", PhonemeType::Vowel), ("t", PhonemeType::Consonant)],
        "can" => vec![("k", PhonemeType::Consonant), ("ae", PhonemeType::Vowel), ("n", PhonemeType::Consonant)],
        "i" => vec![("ai", PhonemeType::Vowel)],
        _ => return None,
    };

    Some(phonemes.into_iter().map(|(s, t)| EnglishPhoneme {
        symbol: s.to_string(),
        phoneme_type: t,
        source: word.to_string(),
    }).collect())
}

/// 영어 음소를 공통 Phoneme 타입으로 변환 (합성 엔진 공유용)
pub fn to_common_phonemes(english_phonemes: &[EnglishPhoneme]) -> Vec<crate::hangul::Phoneme> {
    english_phonemes.iter().map(|ep| {
        crate::hangul::Phoneme {
            symbol: ep.symbol.clone(),
            phoneme_type: ep.phoneme_type.clone(),
            source_char: ep.source.clone(),
        }
    }).collect()
}

/// 영어 음소 포먼트 테이블 (근사값)
pub fn english_vowel_formants(phoneme: &str) -> (f64, f64, f64) {
    match phoneme {
        "iy" | "iy-" => (280.0, 2250.0, 2900.0),
        "ih"  => (400.0, 1920.0, 2560.0),
        "eh"  => (550.0, 1770.0, 2490.0),
        "ae"  => (690.0, 1660.0, 2490.0),
        "aa"  => (710.0, 1100.0, 2540.0),
        "ah"  => (640.0, 1200.0, 2400.0),
        "ao"  => (570.0, 840.0,  2410.0),
        "uh"  => (440.0, 1020.0, 2240.0),
        "uw"  => (300.0, 870.0,  2240.0),
        "er"  => (490.0, 1350.0, 1690.0),
        "ey"  => (500.0, 1700.0, 2600.0),
        "ai"  => (700.0, 1200.0, 2600.0),
        "aw"  => (650.0, 1100.0, 2500.0),
        "ow"  => (500.0, 900.0,  2500.0),
        "oy"  => (550.0, 900.0,  2500.0),
        _ => (500.0, 1500.0, 2500.0), // 기본값
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_english() {
        assert_eq!(normalize_english("Hello World!"), "hello world!");
        assert_eq!(normalize_english("Test 123"), "test onetwothree");
    }

    #[test]
    fn test_dictionary_lookup() {
        let phonemes = english_g2p("hello world");
        let symbols: Vec<&str> = phonemes.iter()
            .filter(|p| !matches!(p.phoneme_type, PhonemeType::Pause))
            .map(|p| p.symbol.as_str())
            .collect();
        assert_eq!(symbols, vec!["hh", "eh", "l", "ow", "w", "er", "l", "d"]);
    }

    #[test]
    fn test_rule_based_g2p() {
        let phonemes = english_g2p("cat");
        let symbols: Vec<&str> = phonemes.iter().map(|p| p.symbol.as_str()).collect();
        assert_eq!(symbols, vec!["k", "ae", "t"]);
    }

    #[test]
    fn test_digraphs() {
        let phonemes = english_g2p("thin");
        let symbols: Vec<&str> = phonemes.iter().map(|p| p.symbol.as_str()).collect();
        assert_eq!(symbols, vec!["th", "ih", "n"]);
    }
}
