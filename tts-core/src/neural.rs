/// 신경망 TTS 엔진 모듈
/// edge-tts (한국어/영어) + piper (영어 로컬 폴백)

use std::io::Read;
use std::process::{Command, Stdio};
use std::path::PathBuf;

pub struct NeuralAudio {
    pub wav_data: Vec<u8>,
    pub sample_rate: u32,
    pub engine: String,
}

/// edge-tts CLI 탐색
fn find_edge_tts() -> Option<String> {
    for cmd in &["edge-tts"] {
        if Command::new(cmd).arg("--help")
            .stdout(Stdio::null()).stderr(Stdio::null())
            .status().map(|s| s.success()).unwrap_or(false)
        {
            return Some(cmd.to_string());
        }
    }
    let home = std::env::var("HOME").unwrap_or_default();
    for ver in &["3.9", "3.10", "3.11", "3.12"] {
        let path = format!("{}/Library/Python/{}/bin/edge-tts", home, ver);
        if std::path::Path::new(&path).exists() {
            return Some(path);
        }
    }
    None
}

/// piper CLI 탐색 (로컬 폴백용)
fn find_piper_bin() -> Option<String> {
    if Command::new("piper").arg("--help")
        .stdout(Stdio::null()).stderr(Stdio::null())
        .status().map(|s| s.success()).unwrap_or(false)
    {
        return Some("piper".into());
    }
    let home = std::env::var("HOME").unwrap_or_default();
    for ver in &["3.9", "3.10", "3.11", "3.12"] {
        let path = format!("{}/Library/Python/{}/bin/piper", home, ver);
        if std::path::Path::new(&path).exists() {
            return Some(path);
        }
    }
    None
}

/// 한국어 합성: edge-tts (ko-KR-SunHiNeural)
pub fn synthesize_korean(text: &str) -> Result<NeuralAudio, String> {
    let edge = find_edge_tts()
        .ok_or("edge-tts가 설치되어 있지 않습니다. `pip3 install edge-tts`를 실행하세요.")?;

    let tmp = std::env::temp_dir().join("tts_edge_ko.mp3");
    let output = Command::new(&edge)
        .args([
            "--voice", "ko-KR-SunHiNeural",
            "--text", text,
            "--write-media", tmp.to_str().unwrap(),
        ])
        .output()
        .map_err(|e| format!("edge-tts 실행 실패: {}", e))?;

    if !output.status.success() {
        return Err(format!("edge-tts 오류: {}", String::from_utf8_lossy(&output.stderr)));
    }

    // MP3 → WAV 변환 (ffmpeg 또는 직접 MP3 반환)
    let mp3_data = std::fs::read(&tmp).map_err(|e| format!("파일 읽기 실패: {}", e))?;
    let _ = std::fs::remove_file(&tmp);

    // ffmpeg가 있으면 WAV로 변환
    if let Some(wav) = mp3_to_wav(&mp3_data) {
        Ok(NeuralAudio { wav_data: wav, sample_rate: 24000, engine: "edge-tts-ko".into() })
    } else {
        // ffmpeg 없으면 MP3 그대로 (브라우저에서 재생 가능)
        Ok(NeuralAudio { wav_data: mp3_data, sample_rate: 24000, engine: "edge-tts-ko-mp3".into() })
    }
}

/// 영어 합성: edge-tts (en-US-AriaNeural)
pub fn synthesize_english(text: &str) -> Result<NeuralAudio, String> {
    let edge = find_edge_tts()
        .ok_or("edge-tts가 설치되어 있지 않습니다. `pip3 install edge-tts`를 실행하세요.")?;

    let tmp = std::env::temp_dir().join("tts_edge_en.mp3");
    let output = Command::new(&edge)
        .args([
            "--voice", "en-US-AriaNeural",
            "--text", text,
            "--write-media", tmp.to_str().unwrap(),
        ])
        .output()
        .map_err(|e| format!("edge-tts 실행 실패: {}", e))?;

    if !output.status.success() {
        return Err(format!("edge-tts 오류: {}", String::from_utf8_lossy(&output.stderr)));
    }

    let mp3_data = std::fs::read(&tmp).map_err(|e| format!("파일 읽기 실패: {}", e))?;
    let _ = std::fs::remove_file(&tmp);

    if let Some(wav) = mp3_to_wav(&mp3_data) {
        Ok(NeuralAudio { wav_data: wav, sample_rate: 24000, engine: "edge-tts-en".into() })
    } else {
        Ok(NeuralAudio { wav_data: mp3_data, sample_rate: 24000, engine: "edge-tts-en-mp3".into() })
    }
}

/// MP3 → WAV 변환 (ffmpeg 사용)
fn mp3_to_wav(mp3: &[u8]) -> Option<Vec<u8>> {
    let tmp_mp3 = std::env::temp_dir().join("tts_tmp.mp3");
    let tmp_wav = std::env::temp_dir().join("tts_tmp.wav");
    std::fs::write(&tmp_mp3, mp3).ok()?;

    let status = Command::new("ffmpeg")
        .args(["-y", "-i", tmp_mp3.to_str()?, "-ar", "24000", "-ac", "1", "-f", "wav", tmp_wav.to_str()?])
        .stdout(Stdio::null()).stderr(Stdio::null())
        .status().ok()?;

    let _ = std::fs::remove_file(&tmp_mp3);

    if status.success() {
        let wav = std::fs::read(&tmp_wav).ok()?;
        let _ = std::fs::remove_file(&tmp_wav);
        Some(wav)
    } else {
        None
    }
}

/// 엔진 상태 확인
pub fn check_engines() -> serde_json::Value {
    let edge_ok = find_edge_tts().is_some();
    let piper_ok = find_piper_bin().is_some();
    let ffmpeg_ok = Command::new("ffmpeg").arg("-version")
        .stdout(Stdio::null()).stderr(Stdio::null())
        .status().map(|s| s.success()).unwrap_or(false);

    serde_json::json!({
        "edge_tts_installed": edge_ok,
        "piper_installed": piper_ok,
        "ffmpeg_installed": ffmpeg_ok,
        "korean_ready": edge_ok,
        "english_ready": edge_ok,
        "audio_format": if ffmpeg_ok { "wav" } else { "mp3" },
        "install_guide": {
            "edge_tts": "pip3 install edge-tts",
            "ffmpeg": "brew install ffmpeg (선택: MP3→WAV 변환용)",
        },
        "voices": {
            "korean": "ko-KR-SunHiNeural",
            "english": "en-US-AriaNeural",
        }
    })
}
