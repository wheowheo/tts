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

/// 간단한 PRNG (xorshift32) — 지터/시머용
struct Rng(u32);
impl Rng {
    fn new(seed: u32) -> Self { Rng(seed) }
    /// -1.0 ~ 1.0 범위의 랜덤 값
    fn next_f64(&mut self) -> f64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        (self.0 as f64 / u32::MAX as f64) * 2.0 - 1.0
    }
}

/// LF(Liljencrants-Fant) 글로탈 펄스 모델
/// Rosenberg 대비: 복귀기(return phase) 추가, 더 풍성한 고조파
fn glottal_pulse_lf(phase: f64) -> f64 {
    let p = phase % 1.0;
    // Te=0.4 (개방기 끝), Tp=0.65 (폐쇄기 끝), Tc=1.0
    if p < 0.4 {
        // 개방기: 사인파 성분 포함 — 더 자연스러운 상승
        let t = p / 0.4;
        let sin_component = (std::f64::consts::PI * t).sin();
        // Hermite-like smooth rise with sinusoidal enrichment
        let poly = t * t * (3.0 - 2.0 * t);
        poly * 0.7 + sin_component * 0.3
    } else if p < 0.65 {
        // 폐쇄기: 급격한 하강 (성대 접촉)
        let t = (p - 0.4) / 0.25;
        let decay = 1.0 - t;
        decay * decay // 2차 감쇠 — Rosenberg보다 자연스러움
    } else if p < 0.85 {
        // 복귀기(return phase): 지수 감쇠로 원위치
        // 이 구간이 Rosenberg에 없던 핵심 — 음색에 따뜻함 추가
        let t = (p - 0.65) / 0.20;
        -0.2 * (-3.0 * t).exp() // 음의 복귀, 지수 감쇠
    } else {
        0.0
    }
}

/// 기식성(aspiration) 노이즈 혼합용 — 자연 음성에는 항상 약간의 기류 노이즈 존재
fn aspiration_noise(rng: &mut Rng) -> f64 {
    rng.next_f64() * 0.03 // 3% 기식 노이즈
}

/// 1차 로우패스 필터 상태 — 스펙트럼 기울기(spectral tilt) 적용용
struct LowPassState {
    prev: f64,
    alpha: f64, // 0.0~1.0, 낮을수록 더 많이 깎음
}

impl LowPassState {
    fn new(cutoff_ratio: f64) -> Self {
        // alpha ≈ cutoff / (cutoff + samplerate/(2π))
        // 간단히: 0.3이면 약 -12dB/octave 효과
        LowPassState { prev: 0.0, alpha: cutoff_ratio }
    }
    fn process(&mut self, input: f64) -> f64 {
        let output = self.alpha * input + (1.0 - self.alpha) * self.prev;
        self.prev = output;
        output
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

/// 유성 자음 합성 공통 헬퍼 (LF 글로탈 + 지터/시머 + 공명)
fn voiced_consonant(num_samples: usize, pitch_hz: f32, amplitude: f32,
                    res_freq: f64, res_bw: f64, seed: u32,
                    attack: f64, release: f64) -> Vec<f64> {
    let f0 = pitch_hz as f64;
    let amp = amplitude as f64 * 0.8;
    let mut samples = Vec::with_capacity(num_samples);
    let mut phase: f64 = 0.0;
    let mut rng = Rng::new(seed);
    let mut prev_phase = 0.0_f64;
    let mut jitter_val = 0.0_f64;
    let mut shimmer_val = 0.0_f64;
    let mut res = ResonatorState::new();
    let mut tilt = LowPassState::new(0.35);
    for i in 0..num_samples {
        let env = envelope(i, num_samples, attack, release);
        if phase.floor() > prev_phase.floor() {
            jitter_val = rng.next_f64() * 0.015;
            shimmer_val = rng.next_f64() * 0.03;
        }
        prev_phase = phase;
        let glottal = glottal_pulse_lf(phase) * (1.0 + shimmer_val);
        let tilted = tilt.process(glottal);
        let out = res.process(tilted, res_freq, res_bw, SAMPLE_RATE as f64);
        samples.push((tilted * 0.3 + out * 3.0) * amp * env);
        phase += (f0 * (1.0 + jitter_val)) / SAMPLE_RATE as f64;
    }
    samples
}

/// 마찰음 스펙트럼 파라미터 (중심 주파수, 대역폭)
fn fricative_band(phoneme: &str) -> (f64, f64) {
    match phoneme {
        "s" | "ss"      => (5500.0, 3000.0), // ㅅ: 고주파 치찰음
        "sh" | "sh-n"   => (3500.0, 2500.0), // sh: 중고주파
        "h" | "hh"      => (1500.0, 2000.0), // ㅎ: 넓은 저주파
        "f"             => (2000.0, 2500.0), // f: 순치 마찰음
        "v"             => (1500.0, 2000.0), // v: 유성 순치
        "z"             => (4500.0, 2500.0), // z: 유성 치찰음
        _               => (3000.0, 2500.0),
    }
}

/// 파열음 VOT(Voice Onset Time) ms — 음소 전체 길이의 비율로
fn plosive_vot_ms(phoneme: &str) -> f64 {
    match phoneme {
        "k" | "t" | "p" | "g" | "d" | "b" => 15.0,
        "kh" | "th" | "ph"                 => 35.0,
        "kk" | "tt" | "pp"                 => 8.0,
        _                                   => 15.0,
    }
}

/// 비음 공명 주파수
fn nasal_resonance(phoneme: &str) -> (f64, f64) {
    match phoneme {
        "m"  => (150.0, 120.0), // 양순: 낮은 공명
        "n"  => (250.0, 100.0), // 치경: 중간
        "ng" => (200.0, 110.0), // 연구개: 중간 낮음
        _    => (250.0, 100.0),
    }
}

/// 자음 합성 (스펙트럼 차별화 + VOT + 개별 공명)
fn synthesize_consonant(phoneme: &str, duration_ms: f32, pitch_hz: f32, amplitude: f32) -> Vec<f64> {
    let num_samples = (duration_ms as f64 / 1000.0 * SAMPLE_RATE as f64) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    let mut rng = Rng::new(12345);
    let amp = amplitude as f64 * 0.8;

    match phoneme {
        // 마찰음: 밴드패스 + 원본 노이즈 블렌드 (볼륨 확보)
        "s" | "ss" | "h" | "hh" | "f" | "v" | "z" | "sh" | "sh-n" => {
            let (center, bw) = fricative_band(phoneme);
            let mut bp = ResonatorState::new();
            let gain = match phoneme {
                "ss" => 1.3,
                _ => 1.0,
            };
            for i in 0..num_samples {
                let env = envelope(i, num_samples, 0.08, 0.12);
                let noise = rng.next_f64();
                let filtered = bp.process(noise, center, bw, SAMPLE_RATE as f64);
                // 밴드패스 출력 + 원본 노이즈 블렌드 (볼륨 확보)
                let mixed = filtered * 5.0 + noise * 0.15;
                samples.push(mixed * amp * env * gain);
            }
        }
        // 파열음: burst → VOT → 잔여 기류 (무음 없음)
        "k" | "kk" | "kh" | "g" | "t" | "tt" | "th" | "d" | "p" | "pp" | "ph" | "b" => {
            let vot = plosive_vot_ms(phoneme);
            let burst_ms = 4.0;
            let burst_samples = (burst_ms / 1000.0 * SAMPLE_RATE as f64) as usize;
            let vot_samples = (vot / 1000.0 * SAMPLE_RATE as f64) as usize;
            let is_aspirated = matches!(phoneme, "kh" | "th" | "ph");
            let is_tense = matches!(phoneme, "kk" | "tt" | "pp");

            let mut bp = ResonatorState::new();
            let burst_freq = match phoneme {
                "k" | "kk" | "kh" | "g" => 1500.0,
                "t" | "tt" | "th" | "d" => 3000.0,
                "p" | "pp" | "ph" | "b" => 800.0,
                _ => 2000.0,
            };

            for i in 0..num_samples {
                if i < burst_samples {
                    let env = envelope(i, burst_samples, 0.1, 0.5);
                    let n = rng.next_f64();
                    let filtered = bp.process(n, burst_freq, 800.0, SAMPLE_RATE as f64);
                    let tense_gain = if is_tense { 1.4 } else { 1.0 };
                    samples.push((filtered * 5.0 + n * 0.3) * amp * env * tense_gain);
                } else if i < burst_samples + vot_samples {
                    let env = envelope(i - burst_samples, vot_samples, 0.15, 0.4);
                    if is_aspirated {
                        let n = rng.next_f64();
                        let filtered = bp.process(n, 2000.0, 2500.0, SAMPLE_RATE as f64);
                        samples.push((filtered * 4.0 + n * 0.15) * amp * env * 0.5);
                    } else {
                        // 평음/경음: 약한 기류 잔향
                        samples.push(rng.next_f64() * amp * env * 0.08);
                    }
                } else {
                    // 잔여 구간: 약한 기류 + 감쇠 (연결감 유지)
                    let tail_pos = (i - burst_samples - vot_samples) as f64
                        / (num_samples - burst_samples - vot_samples).max(1) as f64;
                    let tail_env = (1.0 - tail_pos) * 0.15; // 서서히 감소
                    samples.push(rng.next_f64() * amp * tail_env);
                }
            }
        }
        // 비음: 개별 공명 주파수 + 반공명
        "n" | "m" | "ng" => {
            let (res_freq, res_bw) = nasal_resonance(phoneme);
            // 반공명(anti-resonance) 시뮬레이션: 두 번째 공진기를 역위상으로
            let anti_freq = match phoneme {
                "m" => 1000.0,
                "n" => 1500.0,
                "ng" => 2000.0,
                _ => 1500.0,
            };
            let f0 = pitch_hz as f64;
            let mut phase: f64 = 0.0;
            let mut vrng = Rng::new(7919);
            let mut prev_phase = 0.0_f64;
            let mut jitter_val = 0.0_f64;
            let mut shimmer_val = 0.0_f64;
            let mut res = ResonatorState::new();
            let mut anti_res = ResonatorState::new();
            let mut tilt = LowPassState::new(0.35);
            for i in 0..num_samples {
                let env = envelope(i, num_samples, 0.1, 0.1);
                if phase.floor() > prev_phase.floor() {
                    jitter_val = vrng.next_f64() * 0.015;
                    shimmer_val = vrng.next_f64() * 0.03;
                }
                prev_phase = phase;
                let glottal = glottal_pulse_lf(phase) * (1.0 + shimmer_val);
                let tilted = tilt.process(glottal);
                let nasal = res.process(tilted, res_freq, res_bw, SAMPLE_RATE as f64);
                let anti = anti_res.process(tilted, anti_freq, 200.0, SAMPLE_RATE as f64);
                // 글로탈 소스 직접 믹스 + 공진기 출력 (볼륨 확보)
                samples.push((tilted * 0.3 + nasal * 3.0 - anti * 0.5) * amp * env);
                phase += (f0 * (1.0 + jitter_val)) / SAMPLE_RATE as f64;
            }
        }
        // 반모음/글라이드: 포먼트 전이 시뮬레이션
        "w" => {
            return voiced_consonant(num_samples, pitch_hz, amplitude, 350.0, 100.0, 6151, 0.15, 0.15);
        }
        "y" => {
            return voiced_consonant(num_samples, pitch_hz, amplitude, 1800.0, 150.0, 6151, 0.15, 0.15);
        }
        "kw" => {
            return voiced_consonant(num_samples, pitch_hz, amplitude, 400.0, 120.0, 6151, 0.15, 0.15);
        }
        // 복합 자음 (k-s 등)
        "k-s" => {
            let half = num_samples / 2;
            let mut bp = ResonatorState::new();
            for i in 0..half {
                let env = envelope(i, half, 0.05, 0.5);
                let n = rng.next_f64();
                let filtered = bp.process(n, 1500.0, 800.0, SAMPLE_RATE as f64);
                samples.push((filtered * 5.0 + n * 0.15) * amp * env);
            }
            let mut bp2 = ResonatorState::new();
            for i in 0..(num_samples - half) {
                let env = envelope(i, num_samples - half, 0.1, 0.1);
                let n = rng.next_f64();
                let filtered = bp2.process(n, 5500.0, 3000.0, SAMPLE_RATE as f64);
                samples.push((filtered * 5.0 + n * 0.15) * amp * env);
            }
        }
        // 유음
        "r" => {
            return voiced_consonant(num_samples, pitch_hz, amplitude, 350.0, 80.0, 3571, 0.2, 0.2);
        }
        "l" => {
            return voiced_consonant(num_samples, pitch_hz, amplitude, 400.0, 90.0, 3571, 0.15, 0.15);
        }
        // 파찰음: 버스트 → 마찰 노이즈 (전체 구간 채움)
        "ch" | "j" | "jj" | "jh" => {
            let burst_len = (num_samples as f64 * 0.2) as usize;
            let mut bp = ResonatorState::new();
            for i in 0..num_samples {
                let overall_env = envelope(i, num_samples, 0.05, 0.15);
                if i < burst_len {
                    let env = envelope(i, burst_len, 0.1, 0.3);
                    let noise = rng.next_f64();
                    let filtered = bp.process(noise, 3000.0, 1000.0, SAMPLE_RATE as f64);
                    samples.push((filtered * 5.0 + noise * 0.2) * amp * env);
                } else {
                    let noise = rng.next_f64();
                    let filtered = bp.process(noise, 3500.0, 2500.0, SAMPLE_RATE as f64);
                    samples.push((filtered * 4.0 + noise * 0.1) * amp * overall_env * 0.6);
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

/// 코사인 보간 (0~1 사이에서 부드러운 전이)
fn cosine_interp(a: f64, b: f64, t: f64) -> f64 {
    let ct = (1.0 - (t * std::f64::consts::PI).cos()) * 0.5;
    a * (1.0 - ct) + b * ct
}

/// 포먼트 보간: 전이 구간에서 이전/다음 포먼트 사이를 코사인 보간
fn interpolate_formants(base: &FormantParams, target: &FormantParams, t: f64) -> FormantParams {
    FormantParams {
        f1: cosine_interp(base.f1, target.f1, t),
        f2: cosine_interp(base.f2, target.f2, t),
        f3: cosine_interp(base.f3, target.f3, t),
        bw1: cosine_interp(base.bw1, target.bw1, t),
        bw2: cosine_interp(base.bw2, target.bw2, t),
        bw3: cosine_interp(base.bw3, target.bw3, t),
    }
}

/// 전이 구간의 넓은 대역폭 (포먼트 전이 중 공명이 불안정)
fn transition_bandwidth(base: &FormantParams) -> FormantParams {
    FormantParams {
        bw1: base.bw1 * 1.8,
        bw2: base.bw2 * 1.8,
        bw3: base.bw3 * 1.5,
        ..*base
    }
}

/// 모음 합성: 포먼트 합성 (LF 글로탈 + 지터/시머 + 스펙트럼 기울기 + 포먼트 전이)
fn synthesize_vowel_with_context(
    phoneme: &str,
    duration_ms: f32,
    pitch_hz: f32,
    amplitude: f32,
    prev_formants: Option<&FormantParams>,
    next_formants: Option<&FormantParams>,
) -> Vec<f64> {
    let num_samples = (duration_ms as f64 / 1000.0 * SAMPLE_RATE as f64) as usize;
    let target_formants = vowel_formants(phoneme);
    let f0 = pitch_hz as f64;
    let amp = amplitude as f64;

    // 전이 구간 길이 (샘플 수)
    let onset_ms = if prev_formants.is_some() { 30.0 } else { 0.0 };  // 시작 전이
    let offset_ms = if next_formants.is_some() { 25.0 } else { 0.0 }; // 끝 전이
    let onset_samples = (onset_ms / 1000.0 * SAMPLE_RATE as f64) as usize;
    let offset_samples = (offset_ms / 1000.0 * SAMPLE_RATE as f64) as usize;
    let onset_end = onset_samples.min(num_samples / 3); // 최대 1/3까지
    let offset_start = num_samples.saturating_sub(offset_samples.min(num_samples / 3));

    let mut samples = Vec::with_capacity(num_samples);
    let mut phase: f64 = 0.0;
    let mut rng = Rng::new(phoneme.as_bytes().iter().fold(42u32, |a, &b| a.wrapping_mul(31).wrapping_add(b as u32)));
    let mut res1 = ResonatorState::new();
    let mut res2 = ResonatorState::new();
    let mut res3 = ResonatorState::new();
    let mut tilt = LowPassState::new(0.35);

    let mut jitter_val = 0.0_f64;
    let mut shimmer_val = 0.0_f64;
    let mut prev_phase = 0.0_f64;

    for i in 0..num_samples {
        let env = envelope(i, num_samples, 0.05, 0.1);

        // 글로탈 주기 경계 → 지터/시머 갱신
        let cur_cycle = phase.floor();
        if cur_cycle > prev_phase.floor() {
            jitter_val = rng.next_f64() * 0.015;
            shimmer_val = rng.next_f64() * 0.03;
        }
        prev_phase = phase;

        // 포먼트 전이 계산
        let formants = if i < onset_end && onset_end > 0 {
            // 시작 전이: 이전 음소 → 현재 음소
            let t = i as f64 / onset_end as f64;
            match prev_formants {
                Some(pf) => {
                    let from = transition_bandwidth(pf);
                    interpolate_formants(&from, &target_formants, t)
                }
                None => target_formants,
            }
        } else if i >= offset_start && offset_start < num_samples {
            // 끝 전이: 현재 음소 → 다음 음소
            let t = (i - offset_start) as f64 / (num_samples - offset_start) as f64;
            match next_formants {
                Some(nf) => {
                    let to = transition_bandwidth(nf);
                    interpolate_formants(&target_formants, &to, t)
                }
                None => target_formants,
            }
        } else {
            target_formants
        };

        // LF 글로탈 소스
        let source = glottal_pulse_lf(phase) * (1.0 + shimmer_val)
            + aspiration_noise(&mut rng);
        let source_tilted = tilt.process(source);

        // 포먼트 필터 (보간된 주파수/대역폭 적용)
        let f1_out = res1.process(source_tilted, formants.f1, formants.bw1, SAMPLE_RATE as f64);
        let f2_out = res2.process(source_tilted, formants.f2, formants.bw2, SAMPLE_RATE as f64);
        let f3_out = res3.process(source_tilted, formants.f3, formants.bw3, SAMPLE_RATE as f64);

        let sample = (f1_out * 1.0 + f2_out * 0.7 + f3_out * 0.3) * amp * env;
        samples.push(sample);

        phase += (f0 * (1.0 + jitter_val)) / SAMPLE_RATE as f64;
    }

    samples
}

/// 단독 모음 합성 (테스트/하위 호환용)
#[cfg(test)]
fn synthesize_vowel(phoneme: &str, duration_ms: f32, pitch_hz: f32, amplitude: f32) -> Vec<f64> {
    synthesize_vowel_with_context(phoneme, duration_ms, pitch_hz, amplitude, None, None)
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

/// 음소 타입 조합에 따른 크로스페이드 길이 (ms)
fn crossfade_ms(prev_type: &PhonemeType, cur_type: &PhonemeType) -> f64 {
    match (prev_type, cur_type) {
        (PhonemeType::Consonant, PhonemeType::Vowel) => 20.0,   // C→V
        (PhonemeType::Vowel, PhonemeType::Consonant) => 15.0,   // V→C
        (PhonemeType::Vowel, PhonemeType::Vowel) => 40.0,       // V→V (가장 긴 전이)
        (PhonemeType::Consonant, PhonemeType::Consonant) => 10.0,// C→C
        _ => 5.0,
    }
}

/// 이전 음소에서 모음 포먼트 추출 (포먼트 전이 소스)
fn get_vowel_formants_opt(phoneme: &str, ptype: &PhonemeType) -> Option<FormantParams> {
    match ptype {
        PhonemeType::Vowel => Some(vowel_formants(phoneme)),
        _ => None,
    }
}

/// 전체 운율 시퀀스를 PCM으로 합성 (포먼트 전이 + 타입별 크로스페이드)
pub fn synthesize_all(prosody_units: &[ProsodyUnit]) -> Vec<i16> {
    let mut all_samples: Vec<f64> = Vec::new();
    let mut prev_type = PhonemeType::Silence;

    for (idx, unit) in prosody_units.iter().enumerate() {
        let segment: Vec<f64> = match unit.phoneme_type {
            PhonemeType::Vowel => {
                // 이전/다음 모음 포먼트 조회 (포먼트 전이용)
                let prev_f = if idx > 0 {
                    get_vowel_formants_opt(&prosody_units[idx - 1].phoneme, &prosody_units[idx - 1].phoneme_type)
                } else { None };
                let next_f = if idx + 1 < prosody_units.len() {
                    get_vowel_formants_opt(&prosody_units[idx + 1].phoneme, &prosody_units[idx + 1].phoneme_type)
                } else { None };

                synthesize_vowel_with_context(
                    &unit.phoneme, unit.duration_ms, unit.pitch_hz, unit.amplitude,
                    prev_f.as_ref(), next_f.as_ref(),
                )
            }
            PhonemeType::Consonant => {
                synthesize_consonant(&unit.phoneme, unit.duration_ms, unit.pitch_hz, unit.amplitude)
            }
            PhonemeType::Pause | PhonemeType::Silence => {
                let num_samples = (unit.duration_ms as f64 / 1000.0 * SAMPLE_RATE as f64) as usize;
                vec![0.0; num_samples]
            }
        };

        // 타입별 크로스페이드 적용
        let fade_ms = crossfade_ms(&prev_type, &unit.phoneme_type);
        let fade_len = (SAMPLE_RATE as f64 * fade_ms / 1000.0) as usize;

        if !all_samples.is_empty() && !segment.is_empty() && fade_len > 0 {
            let overlap = fade_len.min(all_samples.len()).min(segment.len());
            let start = all_samples.len() - overlap;
            for j in 0..overlap {
                // 코사인 크로스페이드 (선형보다 부드러움)
                let t = j as f64 / overlap as f64;
                let fade_in = (1.0 - (t * std::f64::consts::PI + std::f64::consts::PI).cos()) * 0.5;
                let fade_out = 1.0 - fade_in;
                all_samples[start + j] = all_samples[start + j] * fade_out + segment[j] * fade_in;
            }
            all_samples.extend_from_slice(&segment[overlap..]);
        } else {
            all_samples.extend_from_slice(&segment);
        }

        prev_type = unit.phoneme_type.clone();
    }

    // 동적 범위 압축 (모음 피크가 자음을 묻지 않도록)
    // 프레임 단위 RMS 기반 소프트 컴프레서
    let frame_size = (SAMPLE_RATE as usize) / 100; // 10ms 프레임
    let mut compressed = all_samples.clone();

    // 1단계: 프레임별 RMS 계산
    let num_frames = (compressed.len() + frame_size - 1) / frame_size;
    let mut frame_rms = Vec::with_capacity(num_frames);
    for f in 0..num_frames {
        let start = f * frame_size;
        let end = (start + frame_size).min(compressed.len());
        let rms = (compressed[start..end].iter().map(|s| s * s).sum::<f64>()
            / (end - start) as f64).sqrt();
        frame_rms.push(rms);
    }

    // 2단계: 전체 평균 RMS 기준으로 압축
    let nonzero_rms: Vec<f64> = frame_rms.iter().filter(|&&r| r > 0.001).cloned().collect();
    if !nonzero_rms.is_empty() {
        let avg_rms = nonzero_rms.iter().sum::<f64>() / nonzero_rms.len() as f64;
        let threshold = avg_rms * 1.5; // 평균의 1.5배 이상이면 압축
        let ratio = 3.0; // 3:1 압축비

        for f in 0..num_frames {
            if frame_rms[f] > threshold {
                // 압축: 초과분을 ratio로 나눔
                let excess = frame_rms[f] / threshold;
                let gain = threshold * (1.0 + (excess - 1.0) / ratio) / frame_rms[f];
                let start = f * frame_size;
                let end = (start + frame_size).min(compressed.len());
                for s in &mut compressed[start..end] {
                    *s *= gain;
                }
            } else if frame_rms[f] > 0.001 && frame_rms[f] < avg_rms * 0.3 {
                // 조용한 구간을 약간 올림 (업워드 컴프레션)
                let boost = (avg_rms * 0.4 / frame_rms[f]).min(2.5);
                let start = f * frame_size;
                let end = (start + frame_size).min(compressed.len());
                for s in &mut compressed[start..end] {
                    *s *= boost;
                }
            }
        }
    }

    // 정규화 및 i16 변환
    let max_val = compressed.iter().map(|s| s.abs()).fold(0.0f64, f64::max);
    let scale = if max_val > 0.001 { 0.8 / max_val } else { 1.0 };

    compressed
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
