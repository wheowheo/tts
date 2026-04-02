/// 운율(Prosody) 생성 모듈
/// 음소 시퀀스에 피치, 길이, 강세 정보를 부여합니다.

use serde::{Deserialize, Serialize};
use crate::hangul::{Phoneme, PhonemeType};

/// 운율 정보가 부여된 음소
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProsodyUnit {
    pub phoneme: String,
    pub phoneme_type: PhonemeType,
    pub duration_ms: f32,
    pub pitch_hz: f32,
    pub amplitude: f32,
    pub source_char: String,
}

/// 운율 파라미터
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProsodyParams {
    pub base_pitch_hz: f32,
    pub speed_factor: f32,
    pub volume: f32,
}

impl Default for ProsodyParams {
    fn default() -> Self {
        ProsodyParams {
            base_pitch_hz: 150.0,
            speed_factor: 1.0,
            volume: 1.0,
        }
    }
}

// 음소별 기본 길이 (밀리초)
fn base_duration(phoneme: &str, ptype: &PhonemeType) -> f32 {
    match ptype {
        PhonemeType::Vowel => match phoneme {
            "a" | "eo" | "o" | "u" | "eu" | "i" => 120.0,
            "ae" | "e" | "oe" | "ui" => 110.0,
            "ya" | "yeo" | "yo" | "yu" | "yae" | "ye" => 130.0,
            "wa" | "wae" | "wo" | "we" | "wi" => 130.0,
            _ => 120.0,
        },
        PhonemeType::Consonant => match phoneme {
            "n" | "m" | "ng" | "l" => 60.0,
            "s" | "ss" | "h" => 80.0,
            "k" | "t" | "p" => 50.0,
            "kk" | "tt" | "pp" | "jj" => 70.0,
            "kh" | "th" | "ph" | "ch" => 80.0,
            "g" | "d" | "b" | "j" => 50.0,
            "r" => 40.0,
            _ => 60.0,
        },
        PhonemeType::Pause => 200.0,
        PhonemeType::Silence => 50.0,
    }
}

/// 문장 내 위치 기반 피치 곡선 (한국어: 문장 끝에서 하강)
fn positional_pitch(base_pitch: f32, position: f32, total: f32) -> f32 {
    let ratio = position / total;
    // 한국어 평서문: 약간 상승 후 문장 끝에서 하강
    let modifier = if ratio < 0.3 {
        1.0 + 0.05 * (ratio / 0.3) // 초반 약간 상승
    } else if ratio < 0.7 {
        1.05 // 중간 유지
    } else {
        1.05 - 0.15 * ((ratio - 0.7) / 0.3) // 후반 하강
    };
    base_pitch * modifier
}

/// 음소 리스트에 운율 정보를 부여
pub fn generate_prosody(phonemes: &[Phoneme], params: &ProsodyParams) -> Vec<ProsodyUnit> {
    let total = phonemes.len() as f32;
    if total == 0.0 {
        return Vec::new();
    }

    phonemes
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let duration = base_duration(&p.symbol, &p.phoneme_type) / params.speed_factor;
            let pitch = positional_pitch(params.base_pitch_hz, i as f32, total);
            let amplitude = params.volume;

            ProsodyUnit {
                phoneme: p.symbol.clone(),
                phoneme_type: p.phoneme_type.clone(),
                duration_ms: duration,
                pitch_hz: pitch,
                amplitude,
                source_char: p.source_char.clone(),
            }
        })
        .collect()
}

/// 어절 경계에 쉼(pause)을 삽입
pub fn insert_pauses(units: &mut Vec<ProsodyUnit>, text: &str) {
    // 간단한 구현: 공백 위치 기반으로 pause 삽입
    // 실제로는 source_char의 변화를 추적하여 어절 경계를 감지
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.len() <= 1 {
        return;
    }

    // 어절 경계 찾기: source_char가 다른 어절에 속하는 위치
    let mut word_boundaries = Vec::new();
    let mut current_word_idx = 0;
    let mut last_char = String::new();

    for (i, unit) in units.iter().enumerate() {
        if unit.source_char != last_char && !last_char.is_empty() {
            // 새 글자로 전환됨
            if current_word_idx < words.len() {
                let current_word = words[current_word_idx];
                if !current_word.contains(&unit.source_char.as_str()) {
                    // 다른 어절로 넘어감
                    word_boundaries.push(i);
                    current_word_idx += 1;
                }
            }
        }
        last_char = unit.source_char.clone();
    }

    // 경계에 pause 삽입 (뒤에서부터 삽입해야 인덱스가 밀리지 않음)
    for boundary in word_boundaries.into_iter().rev() {
        units.insert(boundary, ProsodyUnit {
            phoneme: "pause".into(),
            phoneme_type: PhonemeType::Pause,
            duration_ms: 150.0,
            pitch_hz: 0.0,
            amplitude: 0.0,
            source_char: " ".into(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hangul;

    #[test]
    fn test_generate_prosody() {
        let jamo = hangul::decompose_text("안녕");
        let phonemes = hangul::jamo_to_phonemes(&jamo);
        let params = ProsodyParams::default();
        let prosody = generate_prosody(&phonemes, &params);

        assert_eq!(prosody.len(), phonemes.len());
        for unit in &prosody {
            assert!(unit.duration_ms > 0.0);
            assert!(unit.pitch_hz > 0.0);
            assert!(unit.amplitude > 0.0);
        }
    }

    #[test]
    fn test_pitch_curve() {
        let jamo = hangul::decompose_text("안녕하세요");
        let phonemes = hangul::jamo_to_phonemes(&jamo);
        let params = ProsodyParams::default();
        let prosody = generate_prosody(&phonemes, &params);

        // 문장 끝의 피치가 처음보다 낮아야 함 (하강 곡선)
        let first_pitch = prosody.first().unwrap().pitch_hz;
        let last_pitch = prosody.last().unwrap().pitch_hz;
        assert!(last_pitch < first_pitch + 20.0); // 끝이 시작보다 많이 높지 않아야 함
    }

    #[test]
    fn test_speed_factor() {
        let jamo = hangul::decompose_text("하");
        let phonemes = hangul::jamo_to_phonemes(&jamo);

        let normal = generate_prosody(&phonemes, &ProsodyParams::default());
        let fast = generate_prosody(&phonemes, &ProsodyParams {
            speed_factor: 2.0,
            ..Default::default()
        });

        // 2배속이면 길이가 절반
        assert!((fast[0].duration_ms - normal[0].duration_ms / 2.0).abs() < 0.01);
    }
}
