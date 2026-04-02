/// 신경망 TTS 엔진 모듈
/// Piper(한국어), kokoro-tts(영어)를 subprocess로 호출합니다.

use std::io::Write;
use std::process::{Command, Stdio};
use std::path::PathBuf;

/// 신경망 합성 결과
pub struct NeuralAudio {
    pub wav_data: Vec<u8>,
    pub sample_rate: u32,
    pub engine: String,
}

/// Piper 모델 경로 탐색
fn find_piper_model() -> Option<PathBuf> {
    // 1. 환경변수
    if let Ok(p) = std::env::var("PIPER_KO_MODEL") {
        let path = PathBuf::from(p);
        if path.exists() { return Some(path); }
    }
    // 2. models/ 디렉토리
    for name in &["models/ko-piper.onnx", "models/piper-kss-korean.onnx"] {
        let path = PathBuf::from(name);
        if path.exists() { return Some(path); }
    }
    None
}

/// Piper CLI가 설치되어 있는지 확인
fn piper_available() -> bool {
    Command::new("piper").arg("--help")
        .stdout(Stdio::null()).stderr(Stdio::null())
        .status().map(|s| s.success()).unwrap_or(false)
}

/// 한국어 합성: Piper + KSS 모델
pub fn synthesize_korean(text: &str) -> Result<NeuralAudio, String> {
    if !piper_available() {
        return Err("piper가 설치되어 있지 않습니다. `pip install piper-tts`를 실행하세요.".into());
    }

    let model = find_piper_model()
        .ok_or("한국어 모델을 찾을 수 없습니다. models/ko-piper.onnx를 다운로드하세요.")?;

    let mut child = Command::new("piper")
        .args(["--model", model.to_str().unwrap(), "--output-raw"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("piper 실행 실패: {}", e))?;

    if let Some(ref mut stdin) = child.stdin {
        stdin.write_all(text.as_bytes()).map_err(|e| format!("stdin 쓰기 실패: {}", e))?;
    }
    // stdin 닫기
    drop(child.stdin.take());

    let output = child.wait_with_output().map_err(|e| format!("piper 대기 실패: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("piper 오류: {}", stderr));
    }

    // Piper raw output: 16-bit PCM, 22050Hz, mono
    let pcm = output.stdout;
    let sample_rate = 22050u32;
    let wav = pcm_to_wav(&pcm, sample_rate, 1);

    Ok(NeuralAudio { wav_data: wav, sample_rate, engine: "piper".into() })
}

/// 영어 합성: kokoro-tts CLI 또는 piper 영어 모델
pub fn synthesize_english(text: &str) -> Result<NeuralAudio, String> {
    // kokoro-tts CLI 시도
    if let Ok(status) = Command::new("kokoro-tts").arg("--help")
        .stdout(Stdio::null()).stderr(Stdio::null()).status()
    {
        if status.success() {
            return synthesize_with_kokoro_cli(text);
        }
    }

    // 폴백: piper 영어 모델
    if piper_available() {
        // piper 영어 기본 모델 시도 (pip install로 자동 다운로드)
        return synthesize_with_piper_english(text);
    }

    Err("영어 신경망 TTS가 설치되어 있지 않습니다. `pip install kokoro-tts` 또는 `pip install piper-tts`를 실행하세요.".into())
}

fn synthesize_with_kokoro_cli(text: &str) -> Result<NeuralAudio, String> {
    let tmp = std::env::temp_dir().join("tts_kokoro_output.wav");
    let output = Command::new("kokoro-tts")
        .args([text, tmp.to_str().unwrap(), "--lang", "en"])
        .output()
        .map_err(|e| format!("kokoro-tts 실행 실패: {}", e))?;

    if !output.status.success() {
        return Err(format!("kokoro-tts 오류: {}", String::from_utf8_lossy(&output.stderr)));
    }

    let wav = std::fs::read(&tmp).map_err(|e| format!("WAV 읽기 실패: {}", e))?;
    let _ = std::fs::remove_file(&tmp);

    Ok(NeuralAudio { wav_data: wav, sample_rate: 24000, engine: "kokoro".into() })
}

fn synthesize_with_piper_english(text: &str) -> Result<NeuralAudio, String> {
    // piper 기본 영어 모델 사용
    let mut child = Command::new("piper")
        .args(["--model", "en_US-lessac-medium", "--output-raw"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("piper 실행 실패: {}", e))?;

    if let Some(ref mut stdin) = child.stdin {
        stdin.write_all(text.as_bytes()).map_err(|_| "stdin 쓰기 실패")?;
    }
    drop(child.stdin.take());

    let output = child.wait_with_output().map_err(|e| format!("piper 대기 실패: {}", e))?;
    if !output.status.success() {
        return Err(format!("piper 영어 오류: {}", String::from_utf8_lossy(&output.stderr)));
    }

    let pcm = output.stdout;
    let wav = pcm_to_wav(&pcm, 22050, 1);
    Ok(NeuralAudio { wav_data: wav, sample_rate: 22050, engine: "piper-en".into() })
}

/// Raw PCM을 WAV로 래핑
fn pcm_to_wav(pcm: &[u8], sample_rate: u32, channels: u16) -> Vec<u8> {
    let data_size = pcm.len() as u32;
    let file_size = 36 + data_size;
    let mut wav = Vec::with_capacity(file_size as usize + 8);

    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&file_size.to_le_bytes());
    wav.extend_from_slice(b"WAVE");
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes()); // PCM
    wav.extend_from_slice(&channels.to_le_bytes());
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    let byte_rate = sample_rate * channels as u32 * 2;
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    let block_align = channels * 2;
    wav.extend_from_slice(&block_align.to_le_bytes());
    wav.extend_from_slice(&16u16.to_le_bytes()); // bits per sample
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_size.to_le_bytes());
    wav.extend_from_slice(pcm);

    wav
}

/// 엔진 상태 확인
pub fn check_engines() -> serde_json::Value {
    let piper_ok = piper_available();
    let ko_model = find_piper_model().is_some();
    let kokoro_ok = Command::new("kokoro-tts").arg("--help")
        .stdout(Stdio::null()).stderr(Stdio::null())
        .status().map(|s| s.success()).unwrap_or(false);

    serde_json::json!({
        "piper_installed": piper_ok,
        "korean_model": ko_model,
        "kokoro_installed": kokoro_ok,
        "korean_ready": piper_ok && ko_model,
        "english_ready": kokoro_ok || piper_ok,
        "install_guide": {
            "piper": "pip install piper-tts",
            "korean_model": "mkdir -p models && wget -O models/ko-piper.onnx https://huggingface.co/neurlang/piper-onnx-kss-korean/resolve/main/piper-kss-korean.onnx && wget -O models/ko-piper.onnx.json https://huggingface.co/neurlang/piper-onnx-kss-korean/resolve/main/piper-kss-korean.onnx.json",
            "kokoro": "pip install kokoro-tts"
        }
    })
}
