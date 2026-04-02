/// 로컬 TTS 엔진 모듈
/// macOS `say` 명령 기반 — 설치 불필요, 인터넷 불필요

use std::process::{Command, Stdio};

pub struct NeuralAudio {
    pub wav_data: Vec<u8>,
    pub sample_rate: u32,
    pub engine: String,
}

/// 한국어 음성 목록
const KO_VOICES: &[(&str, &str)] = &[
    ("Yuna", "유나 (기본)"),
    ("Shelley", "Shelley"),
    ("Sandy", "Sandy"),
    ("Reed", "Reed"),
    ("Eddy", "Eddy"),
    ("Flo", "Flo"),
];

/// 영어 음성 목록
const EN_VOICES: &[(&str, &str)] = &[
    ("Samantha", "Samantha (기본)"),
    ("Alex", "Alex"),
    ("Daniel", "Daniel (영국)"),
    ("Karen", "Karen (호주)"),
];

/// 한국어 합성: macOS say + Yuna
pub fn synthesize_korean(text: &str) -> Result<NeuralAudio, String> {
    synthesize_with_say(text, "Yuna", 24000)
}

/// 영어 합성: macOS say + Samantha
pub fn synthesize_english(text: &str) -> Result<NeuralAudio, String> {
    synthesize_with_say(text, "Samantha", 24000)
}

/// 지정 음성으로 합성
pub fn synthesize_with_voice(text: &str, voice: &str) -> Result<NeuralAudio, String> {
    synthesize_with_say(text, voice, 24000)
}

fn synthesize_with_say(text: &str, voice: &str, target_rate: u32) -> Result<NeuralAudio, String> {
    let tmp_aiff = std::env::temp_dir().join("tts_say_output.aiff");
    let tmp_wav = std::env::temp_dir().join("tts_say_output.wav");

    // say → AIFF
    let status = Command::new("say")
        .args(["-v", voice, "-o", tmp_aiff.to_str().unwrap(), text])
        .status()
        .map_err(|e| format!("say 실행 실패: {}", e))?;

    if !status.success() {
        return Err(format!("say 오류 (음성 '{}'를 찾을 수 없습니다)", voice));
    }

    // AIFF → WAV (ffmpeg)
    let ffmpeg_ok = Command::new("ffmpeg")
        .args([
            "-y", "-i", tmp_aiff.to_str().unwrap(),
            "-ar", &target_rate.to_string(),
            "-ac", "1", "-f", "wav",
            tmp_wav.to_str().unwrap(),
        ])
        .stdout(Stdio::null()).stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    let _ = std::fs::remove_file(&tmp_aiff);

    if ffmpeg_ok {
        let wav = std::fs::read(&tmp_wav).map_err(|e| format!("WAV 읽기 실패: {}", e))?;
        let _ = std::fs::remove_file(&tmp_wav);
        Ok(NeuralAudio {
            wav_data: wav,
            sample_rate: target_rate,
            engine: format!("macos-say-{}", voice),
        })
    } else {
        // ffmpeg 없으면 AIFF 직접 반환
        let aiff = std::fs::read(&tmp_aiff).unwrap_or_default();
        let _ = std::fs::remove_file(&tmp_aiff);
        Err("ffmpeg가 필요합니다. `brew install ffmpeg`를 실행하세요.".into())
    }
}

/// 엔진 상태 확인
pub fn check_engines() -> serde_json::Value {
    let say_ok = Command::new("say").arg("--version")
        .stdout(Stdio::null()).stderr(Stdio::null())
        .status().map(|_| true).unwrap_or(false);

    let ffmpeg_ok = Command::new("ffmpeg").arg("-version")
        .stdout(Stdio::null()).stderr(Stdio::null())
        .status().map(|s| s.success()).unwrap_or(false);

    // 사용 가능한 한국어 음성 확인
    let ko_voices: Vec<&str> = if say_ok {
        let output = Command::new("say").arg("-v").arg("?").output().ok();
        if let Some(out) = output {
            let list = String::from_utf8_lossy(&out.stdout);
            KO_VOICES.iter()
                .filter(|(name, _)| list.contains(name))
                .map(|(name, _)| *name)
                .collect()
        } else { vec![] }
    } else { vec![] };

    serde_json::json!({
        "say_available": say_ok,
        "ffmpeg_installed": ffmpeg_ok,
        "korean_ready": say_ok && ffmpeg_ok && !ko_voices.is_empty(),
        "english_ready": say_ok && ffmpeg_ok,
        "korean_voices": KO_VOICES.iter().map(|(n, d)| serde_json::json!({"id": n, "name": d})).collect::<Vec<_>>(),
        "english_voices": EN_VOICES.iter().map(|(n, d)| serde_json::json!({"id": n, "name": d})).collect::<Vec<_>>(),
        "type": "local",
        "install_guide": if ffmpeg_ok { "설치 완료" } else { "brew install ffmpeg" },
    })
}
