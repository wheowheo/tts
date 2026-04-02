/// 파형 합성(Formant Synthesis) 모듈
/// 운율 정보를 기반으로 실제 PCM 오디오를 생성합니다.

use crate::prosody::ProsodyUnit;
use crate::hangul::PhonemeType;

const SAMPLE_RATE: u32 = 44100;
const PI2: f64 = std::f64::consts::PI * 2.0;

/// 포먼트 파라미터 (F1, F2, F3 주파수 및 대역폭)
#[derive(Debug, Clone, Copy)]
struct FormantParams {
    f1: f64,
    f2: f64,
    f3: f64,
    bw1: f64,
    bw2: f64,
    bw3: f64,
}

/// 한국어 모음 포먼트 테이블 (근사값, Hz)
fn vowel_formants(phoneme: &str) -> FormantParams {
    match phoneme {
        "a"   => FormantParams { f1: 800.0, f2: 1200.0, f3: 2600.0, bw1: 80.0, bw2: 90.0, bw3: 120.0 },
        "ae"  => FormantParams { f1: 700.0, f2: 1800.0, f3: 2600.0, bw1: 80.0, bw2: 90.0, bw3: 120.0 },
        "eo"  => FormantParams { f1: 600.0, f2: 1000.0, f3: 2600.0, bw1: 80.0, bw2: 90.0, bw3: 120.0 },
        "e"   => FormantParams { f1: 550.0, f2: 1800.0, f3: 2600.0, bw1: 80.0, bw2: 90.0, bw3: 120.0 },
        "o"   => FormantParams { f1: 500.0, f2: 900.0,  f3: 2600.0, bw1: 80.0, bw2: 90.0, bw3: 120.0 },
        "u"   => FormantParams { f1: 350.0, f2: 800.0,  f3: 2600.0, bw1: 80.0, bw2: 90.0, bw3: 120.0 },
        "eu"  => FormantParams { f1: 400.0, f2: 1300.0, f3: 2600.0, bw1: 80.0, bw2: 90.0, bw3: 120.0 },
        "i"   => FormantParams { f1: 300.0, f2: 2200.0, f3: 2900.0, bw1: 80.0, bw2: 90.0, bw3: 120.0 },
        // 이중 모음 (근사: 첫 요소 + 두 번째 요소 블렌드)
        "ya"  => FormantParams { f1: 750.0, f2: 1400.0, f3: 2700.0, bw1: 80.0, bw2: 90.0, bw3: 120.0 },
        "yeo" => FormantParams { f1: 550.0, f2: 1200.0, f3: 2600.0, bw1: 80.0, bw2: 90.0, bw3: 120.0 },
        "yo"  => FormantParams { f1: 450.0, f2: 1000.0, f3: 2600.0, bw1: 80.0, bw2: 90.0, bw3: 120.0 },
        "yu"  => FormantParams { f1: 320.0, f2: 1000.0, f3: 2600.0, bw1: 80.0, bw2: 90.0, bw3: 120.0 },
        "ye"  => FormantParams { f1: 500.0, f2: 2000.0, f3: 2700.0, bw1: 80.0, bw2: 90.0, bw3: 120.0 },
        "yae" => FormantParams { f1: 650.0, f2: 1900.0, f3: 2700.0, bw1: 80.0, bw2: 90.0, bw3: 120.0 },
        "wa"  => FormantParams { f1: 700.0, f2: 1100.0, f3: 2600.0, bw1: 80.0, bw2: 90.0, bw3: 120.0 },
        "wae" => FormantParams { f1: 650.0, f2: 1700.0, f3: 2600.0, bw1: 80.0, bw2: 90.0, bw3: 120.0 },
        "wo"  => FormantParams { f1: 550.0, f2: 950.0,  f3: 2600.0, bw1: 80.0, bw2: 90.0, bw3: 120.0 },
        "we"  => FormantParams { f1: 500.0, f2: 1750.0, f3: 2600.0, bw1: 80.0, bw2: 90.0, bw3: 120.0 },
        "wi"  => FormantParams { f1: 350.0, f2: 2000.0, f3: 2800.0, bw1: 80.0, bw2: 90.0, bw3: 120.0 },
        "oe"  => FormantParams { f1: 450.0, f2: 1600.0, f3: 2600.0, bw1: 80.0, bw2: 90.0, bw3: 120.0 },
        "ui"  => FormantParams { f1: 350.0, f2: 1800.0, f3: 2700.0, bw1: 80.0, bw2: 90.0, bw3: 120.0 },
        // 영어 모음
        "iy"  => FormantParams { f1: 280.0, f2: 2250.0, f3: 2900.0, bw1: 80.0, bw2: 90.0, bw3: 120.0 },
        "ih"  => FormantParams { f1: 400.0, f2: 1920.0, f3: 2560.0, bw1: 80.0, bw2: 90.0, bw3: 120.0 },
        "eh"  => FormantParams { f1: 550.0, f2: 1770.0, f3: 2490.0, bw1: 80.0, bw2: 90.0, bw3: 120.0 },
        "aa"  => FormantParams { f1: 710.0, f2: 1100.0, f3: 2540.0, bw1: 80.0, bw2: 90.0, bw3: 120.0 },
        "ah"  => FormantParams { f1: 640.0, f2: 1200.0, f3: 2400.0, bw1: 80.0, bw2: 90.0, bw3: 120.0 },
        "ao"  => FormantParams { f1: 570.0, f2: 840.0,  f3: 2410.0, bw1: 80.0, bw2: 90.0, bw3: 120.0 },
        "uh"  => FormantParams { f1: 440.0, f2: 1020.0, f3: 2240.0, bw1: 80.0, bw2: 90.0, bw3: 120.0 },
        "uw"  => FormantParams { f1: 300.0, f2: 870.0,  f3: 2240.0, bw1: 80.0, bw2: 90.0, bw3: 120.0 },
        "er"  => FormantParams { f1: 490.0, f2: 1350.0, f3: 1690.0, bw1: 80.0, bw2: 90.0, bw3: 120.0 },
        "ey"  => FormantParams { f1: 500.0, f2: 1700.0, f3: 2600.0, bw1: 80.0, bw2: 90.0, bw3: 120.0 },
        "ai"  => FormantParams { f1: 700.0, f2: 1200.0, f3: 2600.0, bw1: 80.0, bw2: 90.0, bw3: 120.0 },
        "aw"  => FormantParams { f1: 650.0, f2: 1100.0, f3: 2500.0, bw1: 80.0, bw2: 90.0, bw3: 120.0 },
        "ow"  => FormantParams { f1: 500.0, f2: 900.0,  f3: 2500.0, bw1: 80.0, bw2: 90.0, bw3: 120.0 },
        "oy"  => FormantParams { f1: 550.0, f2: 900.0,  f3: 2500.0, bw1: 80.0, bw2: 90.0, bw3: 120.0 },
        // 복합 영어 모음 (하이픈 포함)
        "ah-s"  => FormantParams { f1: 640.0, f2: 1200.0, f3: 2400.0, bw1: 80.0, bw2: 90.0, bw3: 120.0 },
        "ih-ng" => FormantParams { f1: 400.0, f2: 1920.0, f3: 2560.0, bw1: 80.0, bw2: 90.0, bw3: 120.0 },
        "aa-r"  => FormantParams { f1: 710.0, f2: 1100.0, f3: 2540.0, bw1: 80.0, bw2: 90.0, bw3: 120.0 },
        "ao-r"  => FormantParams { f1: 570.0, f2: 840.0,  f3: 2410.0, bw1: 80.0, bw2: 90.0, bw3: 120.0 },
        "ao-l"  => FormantParams { f1: 570.0, f2: 840.0,  f3: 2410.0, bw1: 80.0, bw2: 90.0, bw3: 120.0 },
        _ => FormantParams { f1: 500.0, f2: 1500.0, f3: 2600.0, bw1: 80.0, bw2: 90.0, bw3: 120.0 },
    }
}

/// 글로탈 펄스 파형 생성 (성대 진동 시뮬레이션)
fn glottal_pulse(phase: f64) -> f64 {
    // Rosenberg 모델 근사
    let p = phase % 1.0;
    if p < 0.4 {
        // 개방기: 상승
        let t = p / 0.4;
        3.0 * t * t - 2.0 * t * t * t
    } else if p < 0.6 {
        // 폐쇄기: 하강
        let t = (p - 0.4) / 0.2;
        1.0 - t * t
    } else {
        0.0
    }
}

/// 포먼트 공진기 (2차 IIR 필터)
struct ResonatorState {
    y1: f64,
    y2: f64,
}

impl ResonatorState {
    fn new() -> Self {
        ResonatorState { y1: 0.0, y2: 0.0 }
    }

    fn process(&mut self, input: f64, freq: f64, bandwidth: f64, sample_rate: f64) -> f64 {
        let r = (-std::f64::consts::PI * bandwidth / sample_rate).exp();
        let theta = PI2 * freq / sample_rate;
        let a1 = -2.0 * r * theta.cos();
        let a2 = r * r;
        let gain = 1.0 - r;

        let output = gain * input - a1 * self.y1 - a2 * self.y2;
        self.y2 = self.y1;
        self.y1 = output;
        output
    }
}

/// 자음 합성: 노이즈 기반
fn synthesize_consonant(phoneme: &str, duration_ms: f32, pitch_hz: f32, amplitude: f32) -> Vec<f64> {
    let num_samples = (duration_ms as f64 / 1000.0 * SAMPLE_RATE as f64) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    let mut rng_state: u32 = 12345;

    // 간단한 PRNG (xorshift)
    let mut noise = || -> f64 {
        rng_state ^= rng_state << 13;
        rng_state ^= rng_state >> 17;
        rng_state ^= rng_state << 5;
        (rng_state as f64 / u32::MAX as f64) * 2.0 - 1.0
    };

    let amp = amplitude as f64 * 0.3;

    match phoneme {
        // 마찰음: 지속적인 노이즈
        "s" | "ss" | "h" | "hh" | "f" | "v" | "z" | "sh" | "sh-n" => {
            for i in 0..num_samples {
                let env = envelope(i, num_samples, 0.1, 0.1);
                samples.push(noise() * amp * env);
            }
        }
        // 파열음: 짧은 버스트 + 무음
        "k" | "kk" | "kh" | "g" | "t" | "tt" | "th" | "d" | "p" | "pp" | "ph" | "b" => {
            let burst_len = (num_samples as f64 * 0.3) as usize;
            for i in 0..num_samples {
                if i < burst_len {
                    let env = envelope(i, burst_len, 0.05, 0.5);
                    samples.push(noise() * amp * env * 1.5);
                } else {
                    samples.push(0.0);
                }
            }
        }
        // 비음: 성대 진동 + 비강 공명
        "n" | "m" | "ng" => {
            let f0 = pitch_hz as f64;
            let mut phase = 0.0;
            let mut res = ResonatorState::new();
            for i in 0..num_samples {
                let env = envelope(i, num_samples, 0.1, 0.1);
                let glottal = glottal_pulse(phase);
                let nasal = res.process(glottal, 250.0, 100.0, SAMPLE_RATE as f64);
                samples.push(nasal * amp * env * 2.0);
                phase += f0 / SAMPLE_RATE as f64;
            }
        }
        // 반모음/글라이드
        "w" | "y" | "kw" => {
            let f0 = pitch_hz as f64;
            let mut phase = 0.0;
            let mut res = ResonatorState::new();
            for i in 0..num_samples {
                let env = envelope(i, num_samples, 0.15, 0.15);
                let glottal = glottal_pulse(phase);
                let out = res.process(glottal, 400.0, 100.0, SAMPLE_RATE as f64);
                samples.push(out * amp * env * 2.0);
                phase += f0 / SAMPLE_RATE as f64;
            }
        }
        // 복합 자음 (k-s 등)
        "k-s" => {
            let half = num_samples / 2;
            for i in 0..half {
                let env = envelope(i, half, 0.05, 0.5);
                samples.push(noise() * amp * env * 1.5);
            }
            for i in 0..(num_samples - half) {
                let env = envelope(i, num_samples - half, 0.1, 0.1);
                samples.push(noise() * amp * env);
            }
        }
        // 유음
        "r" | "l" => {
            let f0 = pitch_hz as f64;
            let mut phase = 0.0;
            let mut res = ResonatorState::new();
            for i in 0..num_samples {
                let env = envelope(i, num_samples, 0.15, 0.15);
                let glottal = glottal_pulse(phase);
                let lateral = res.process(glottal, 350.0, 80.0, SAMPLE_RATE as f64);
                samples.push(lateral * amp * env * 2.0);
                phase += f0 / SAMPLE_RATE as f64;
            }
        }
        // 파찰음
        "ch" | "j" | "jj" | "jh" => {
            let burst_len = (num_samples as f64 * 0.4) as usize;
            for i in 0..num_samples {
                if i < burst_len {
                    let env = envelope(i, burst_len, 0.1, 0.3);
                    samples.push(noise() * amp * env);
                } else {
                    samples.push(0.0);
                }
            }
        }
        _ => {
            for _ in 0..num_samples {
                samples.push(0.0);
            }
        }
    }

    samples
}

/// 모음 합성: 포먼트 합성
fn synthesize_vowel(phoneme: &str, duration_ms: f32, pitch_hz: f32, amplitude: f32) -> Vec<f64> {
    let num_samples = (duration_ms as f64 / 1000.0 * SAMPLE_RATE as f64) as usize;
    let formants = vowel_formants(phoneme);
    let f0 = pitch_hz as f64;
    let amp = amplitude as f64;

    let mut samples = Vec::with_capacity(num_samples);
    let mut phase = 0.0;
    let mut res1 = ResonatorState::new();
    let mut res2 = ResonatorState::new();
    let mut res3 = ResonatorState::new();

    for i in 0..num_samples {
        let env = envelope(i, num_samples, 0.05, 0.1);

        // 글로탈 소스
        let source = glottal_pulse(phase);

        // 포먼트 필터 적용 (병렬 합성)
        let f1_out = res1.process(source, formants.f1, formants.bw1, SAMPLE_RATE as f64);
        let f2_out = res2.process(source, formants.f2, formants.bw2, SAMPLE_RATE as f64);
        let f3_out = res3.process(source, formants.f3, formants.bw3, SAMPLE_RATE as f64);

        let sample = (f1_out * 1.0 + f2_out * 0.7 + f3_out * 0.3) * amp * env;
        samples.push(sample);

        phase += f0 / SAMPLE_RATE as f64;
    }

    samples
}

/// ADSR 엔벨로프 (attack/release 비율)
fn envelope(sample_idx: usize, total_samples: usize, attack_ratio: f64, release_ratio: f64) -> f64 {
    let pos = sample_idx as f64 / total_samples as f64;
    if pos < attack_ratio {
        pos / attack_ratio
    } else if pos > (1.0 - release_ratio) {
        (1.0 - pos) / release_ratio
    } else {
        1.0
    }
}

/// 전체 운율 시퀀스를 PCM으로 합성
pub fn synthesize_all(prosody_units: &[ProsodyUnit]) -> Vec<i16> {
    let mut all_samples: Vec<f64> = Vec::new();
    let fade_len = (SAMPLE_RATE as f64 * 0.005) as usize; // 5ms 크로스페이드

    for unit in prosody_units {
        let segment: Vec<f64> = match unit.phoneme_type {
            PhonemeType::Vowel => {
                synthesize_vowel(&unit.phoneme, unit.duration_ms, unit.pitch_hz, unit.amplitude)
            }
            PhonemeType::Consonant => {
                synthesize_consonant(&unit.phoneme, unit.duration_ms, unit.pitch_hz, unit.amplitude)
            }
            PhonemeType::Pause | PhonemeType::Silence => {
                let num_samples = (unit.duration_ms as f64 / 1000.0 * SAMPLE_RATE as f64) as usize;
                vec![0.0; num_samples]
            }
        };

        // 크로스페이드 적용
        if !all_samples.is_empty() && !segment.is_empty() && fade_len > 0 {
            let overlap = fade_len.min(all_samples.len()).min(segment.len());
            let start = all_samples.len() - overlap;
            for j in 0..overlap {
                let fade_out = 1.0 - (j as f64 / overlap as f64);
                let fade_in = j as f64 / overlap as f64;
                all_samples[start + j] = all_samples[start + j] * fade_out + segment[j] * fade_in;
            }
            all_samples.extend_from_slice(&segment[overlap..]);
        } else {
            all_samples.extend_from_slice(&segment);
        }
    }

    // 정규화 및 i16 변환
    let max_val = all_samples.iter().map(|s| s.abs()).fold(0.0f64, f64::max);
    let scale = if max_val > 0.001 { 0.8 / max_val } else { 1.0 };

    all_samples
        .iter()
        .map(|s| (s * scale * i16::MAX as f64) as i16)
        .collect()
}

/// PCM 데이터를 WAV 형식으로 인코딩
pub fn encode_wav(samples: &[i16], sample_rate: u32, channels: u16) -> Vec<u8> {
    let data_size = (samples.len() * 2) as u32;
    let file_size = 36 + data_size;

    let mut wav = Vec::with_capacity(file_size as usize + 8);

    // RIFF header
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&file_size.to_le_bytes());
    wav.extend_from_slice(b"WAVE");

    // fmt chunk
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes()); // chunk size
    wav.extend_from_slice(&1u16.to_le_bytes());  // PCM format
    wav.extend_from_slice(&channels.to_le_bytes());
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    let byte_rate = sample_rate * channels as u32 * 2;
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    let block_align = channels * 2;
    wav.extend_from_slice(&block_align.to_le_bytes());
    wav.extend_from_slice(&16u16.to_le_bytes()); // bits per sample

    // data chunk
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_size.to_le_bytes());
    for sample in samples {
        wav.extend_from_slice(&sample.to_le_bytes());
    }

    wav
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{hangul, prosody};

    #[test]
    fn test_synthesize_vowel() {
        let samples = synthesize_vowel("a", 100.0, 150.0, 1.0);
        assert!(!samples.is_empty());
        assert!(samples.iter().any(|s| s.abs() > 0.01));
    }

    #[test]
    fn test_synthesize_consonant() {
        let samples = synthesize_consonant("s", 80.0, 150.0, 1.0);
        assert!(!samples.is_empty());
        assert!(samples.iter().any(|s| s.abs() > 0.01));
    }

    #[test]
    fn test_synthesize_pipeline() {
        let jamo = hangul::decompose_text("안녕");
        let phonemes = hangul::jamo_to_phonemes(&jamo);
        let params = prosody::ProsodyParams::default();
        let prosody_units = prosody::generate_prosody(&phonemes, &params);
        let pcm = synthesize_all(&prosody_units);

        assert!(!pcm.is_empty());
        assert!(pcm.iter().any(|s| *s != 0));
    }

    #[test]
    fn test_encode_wav() {
        let samples = vec![0i16, 1000, -1000, 0];
        let wav = encode_wav(&samples, 44100, 1);

        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
        assert_eq!(&wav[36..40], b"data");
    }

    #[test]
    fn test_full_pipeline_wav() {
        let jamo = hangul::decompose_text("안녕하세요");
        let phonemes = hangul::jamo_to_phonemes(&jamo);
        let params = prosody::ProsodyParams::default();
        let prosody_units = prosody::generate_prosody(&phonemes, &params);
        let pcm = synthesize_all(&prosody_units);
        let wav = encode_wav(&pcm, 44100, 1);

        assert!(wav.len() > 44); // WAV 헤더 + 데이터
        assert_eq!(&wav[0..4], b"RIFF");
    }
}
