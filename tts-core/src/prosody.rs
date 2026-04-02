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
        PhonemeType::Pause => 100.0,
        PhonemeType::Silence => 30.0,
    }
}

/// 음소 타입별 자연 진폭 계수
fn natural_amplitude(ptype: &PhonemeType, phoneme: &str) -> f32 {
    match ptype {
        PhonemeType::Vowel => 1.0,
        PhonemeType::Consonant => match phoneme {
            "n" | "m" | "ng" => 0.85,
            "r" | "l" => 0.80,
            "w" | "y" => 0.90,
            "s" | "ss" | "sh" => 0.75,
            "h" | "hh" => 0.70,
            "f" | "v" | "z" => 0.70,
            "k" | "t" | "p" | "g" | "d" | "b" => 0.80,
            "kk" | "tt" | "pp" => 0.85,
            "kh" | "th" | "ph" => 0.75,
            "ch" | "j" | "jj" | "jh" => 0.75,
            _ => 0.75,
        },
        PhonemeType::Pause | PhonemeType::Silence => 0.0,
    }
}

/// 연속 피치 곡선 생성 — 문장 전체에 걸쳐 부드러운 곡선
/// 반환: 각 음소에 대한 피치 값 배열
fn generate_pitch_contour(num_phonemes: usize, base_pitch: f32) -> Vec<f32> {
    if num_phonemes == 0 {
        return Vec::new();
    }
    if num_phonemes == 1 {
        return vec![base_pitch];
    }

    let mut pitches = Vec::with_capacity(num_phonemes);
    let n = num_phonemes as f32;

    for i in 0..num_phonemes {
        let ratio = i as f32 / (n - 1.0);

        // 한국어 평서문 억양: 부드러운 곡선
        // 시작(0.0) → 상승(0.2) → 피크(0.35) → 완만 유지(0.6) → 하강(1.0)
        let modifier = if ratio < 0.2 {
            // 초반 상승 (코사인 보간)
            let t = ratio / 0.2;
            1.0 + 0.08 * (1.0 - (t * std::f32::consts::PI + std::f32::consts::PI).cos()) * 0.5
        } else if ratio < 0.35 {
            // 피크 근처
            let t = (ratio - 0.2) / 0.15;
            1.08 + 0.02 * (1.0 - (t * std::f32::consts::PI).cos()) * 0.5
        } else if ratio < 0.6 {
            // 완만한 유지/미세 하강
            let t = (ratio - 0.35) / 0.25;
            1.10 - 0.03 * t
        } else {
            // 후반 하강 (코사인 감쇠)
            let t = (ratio - 0.6) / 0.4;
            let descent = (1.0 - (t * std::f32::consts::PI + std::f32::consts::PI).cos()) * 0.5;
            1.07 - 0.17 * descent
        };

        pitches.push(base_pitch * modifier);
    }

    // 피치 스무딩: 3-point 이동 평균
    if num_phonemes >= 3 {
        let orig = pitches.clone();
        for i in 1..num_phonemes - 1 {
            pitches[i] = orig[i - 1] * 0.25 + orig[i] * 0.5 + orig[i + 1] * 0.25;
        }
    }

    pitches
}

/// 문맥 기반 음소 길이 조정
fn contextual_duration(
    base_dur: f32,
    idx: usize,
    total: usize,
    ptype: &PhonemeType,
    phonemes: &[Phoneme],
) -> f32 {
    let mut dur = base_dur;

    match ptype {
        PhonemeType::Vowel => {
            // 어절/문장 마지막 모음: phrase-final lengthening (+25%)
            let is_last_vowel = (idx + 1..total).all(|j| {
                !matches!(phonemes[j].phoneme_type, PhonemeType::Vowel)
            });
            if is_last_vowel {
                dur *= 1.25;
            }

            // 마지막에서 두 번째 모음도 약간 늘림 (+10%)
            let vowels_after: usize = (idx + 1..total)
                .filter(|&j| matches!(phonemes[j].phoneme_type, PhonemeType::Vowel))
                .count();
            if vowels_after == 1 {
                dur *= 1.10;
            }

            // 종성(다음이 자음) 앞 모음: 약간 줄임 (-8%)
            if idx + 1 < total && matches!(phonemes[idx + 1].phoneme_type, PhonemeType::Consonant) {
                dur *= 0.92;
            }
        }
        PhonemeType::Consonant => {
            // 어두 자음: 약간 늘림 (+10%)
            if idx == 0 || (idx > 0 && matches!(phonemes[idx - 1].phoneme_type, PhonemeType::Pause)) {
                dur *= 1.10;
            }
        }
        _ => {}
    }

    dur
}

/// 음소 리스트에 운율 정보를 부여 (개선: 연속 피치 + 문맥 길이 + 진폭 곡선)
pub fn generate_prosody(phonemes: &[Phoneme], params: &ProsodyParams) -> Vec<ProsodyUnit> {
    let total = phonemes.len();
    if total == 0 {
        return Vec::new();
    }

    // 연속 피치 곡선 생성
    let pitch_contour = generate_pitch_contour(total, params.base_pitch_hz);

    phonemes
        .iter()
        .enumerate()
        .map(|(i, p)| {
            // 문맥 기반 길이
            let base_dur = base_duration(&p.symbol, &p.phoneme_type);
            let ctx_dur = contextual_duration(base_dur, i, total, &p.phoneme_type, phonemes);
            let duration = ctx_dur / params.speed_factor;

            // 연속 피치 곡선에서 가져옴
            let pitch = pitch_contour[i];

            // 진폭: 타입별 자연 계수 × 위치 기반 감쇠
            let type_amp = natural_amplitude(&p.phoneme_type, &p.symbol);
            let pos_ratio = i as f32 / total as f32;
            // 문장 끝으로 갈수록 점진적 감소 (마지막 20%에서)
            let pos_amp = if pos_ratio > 0.8 {
                1.0 - 0.15 * ((pos_ratio - 0.8) / 0.2)
            } else {
                1.0
            };
            let amplitude = params.volume * type_amp * pos_amp;

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
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.len() <= 1 {
        return;
    }

    let mut word_boundaries = Vec::new();
    let mut current_word_idx = 0;
    let mut last_char = String::new();

    for (i, unit) in units.iter().enumerate() {
        if unit.source_char != last_char && !last_char.is_empty() {
            if current_word_idx < words.len() {
                let current_word = words[current_word_idx];
                if !current_word.contains(&unit.source_char.as_str()) {
                    word_boundaries.push(i);
                    current_word_idx += 1;
                }
            }
        }
        last_char = unit.source_char.clone();
    }

    for boundary in word_boundaries.into_iter().rev() {
        units.insert(boundary, ProsodyUnit {
            phoneme: "pause".into(),
            phoneme_type: PhonemeType::Pause,
            duration_ms: 100.0,
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
        }
    }

    #[test]
    fn test_pitch_curve_smooth() {
        let jamo = hangul::decompose_text("안녕하세요");
        let phonemes = hangul::jamo_to_phonemes(&jamo);
        let params = ProsodyParams::default();
        let prosody = generate_prosody(&phonemes, &params);

        // 피치가 부드럽게 변해야 함 (인접 음소 간 큰 점프 없음)
        for i in 1..prosody.len() {
            if prosody[i].pitch_hz > 0.0 && prosody[i - 1].pitch_hz > 0.0 {
                let diff = (prosody[i].pitch_hz - prosody[i - 1].pitch_hz).abs();
                assert!(diff < 15.0, "피치 점프가 너무 큼: {}Hz at {}", diff, i);
            }
        }

        // 문장 끝의 피치가 시작보다 낮아야 함
        let first = prosody.first().unwrap().pitch_hz;
        let last = prosody.last().unwrap().pitch_hz;
        assert!(last < first + 5.0, "끝 피치({})가 시작({})보다 높음", last, first);
    }

    #[test]
    fn test_amplitude_variation() {
        let jamo = hangul::decompose_text("안녕하세요");
        let phonemes = hangul::jamo_to_phonemes(&jamo);
        let params = ProsodyParams::default();
        let prosody = generate_prosody(&phonemes, &params);

        // 모음과 자음의 진폭이 달라야 함
        let vowel_amps: Vec<f32> = prosody.iter()
            .filter(|u| matches!(u.phoneme_type, PhonemeType::Vowel))
            .map(|u| u.amplitude)
            .collect();
        let cons_amps: Vec<f32> = prosody.iter()
            .filter(|u| matches!(u.phoneme_type, PhonemeType::Consonant))
            .map(|u| u.amplitude)
            .collect();

        if !vowel_amps.is_empty() && !cons_amps.is_empty() {
            let avg_v: f32 = vowel_amps.iter().sum::<f32>() / vowel_amps.len() as f32;
            let avg_c: f32 = cons_amps.iter().sum::<f32>() / cons_amps.len() as f32;
            assert!(avg_v > avg_c, "모음 진폭({})이 자음({})보다 커야 함", avg_v, avg_c);
        }
    }

    #[test]
    fn test_phrase_final_lengthening() {
        let jamo = hangul::decompose_text("하");
        let phonemes = hangul::jamo_to_phonemes(&jamo);
        let params = ProsodyParams::default();
        let prosody = generate_prosody(&phonemes, &params);

        // 마지막 모음이 기본값보다 길어야 함 (phrase-final lengthening)
        let last_vowel = prosody.iter().rev()
            .find(|u| matches!(u.phoneme_type, PhonemeType::Vowel));
        if let Some(v) = last_vowel {
            let base = base_duration(&v.phoneme, &PhonemeType::Vowel);
            assert!(v.duration_ms >= base, "마지막 모음 길이({})가 기본({})보다 짧음", v.duration_ms, base);
        }
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

        // 2배속이면 길이 비율이 약 2:1
        let ratio = normal[0].duration_ms / fast[0].duration_ms;
        assert!((ratio - 2.0).abs() < 0.1, "속도 비율({})이 2.0이 아님", ratio);
    }
}
