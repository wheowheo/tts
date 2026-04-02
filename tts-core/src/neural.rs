/// 신경망 TTS 엔진 모듈
/// Coqui TTS (VITS) — 한국어/영어 사전학습 모델 + 커스텀 학습

use std::process::{Command, Stdio};
use std::path::{Path, PathBuf};

pub struct NeuralAudio {
    pub wav_data: Vec<u8>,
    pub sample_rate: u32,
    pub engine: String,
}

/// venv 내 tts CLI 경로
fn tts_bin() -> PathBuf {
    PathBuf::from(".venv/bin/tts")
}

fn tts_available() -> bool {
    tts_bin().exists()
}

/// 사전학습 모델로 합성
pub fn synthesize(text: &str, language: &str) -> Result<NeuralAudio, String> {
    if !tts_available() {
        return Err("Coqui TTS가 설치되어 있지 않습니다. setup_neural.sh를 실행하세요.".into());
    }

    let model_name = match language {
        "ko" => "tts_models/kor/fairseq/vits",
        "en" => "tts_models/en/ljspeech/vits",
        _ => "tts_models/kor/fairseq/vits",
    };

    let tmp = std::env::temp_dir().join("tts_neural_output.wav");

    let output = Command::new(tts_bin())
        .args([
            "--text", text,
            "--model_name", model_name,
            "--out_path", tmp.to_str().unwrap(),
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("tts 실행 실패: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("tts 오류: {}", stderr.lines().last().unwrap_or(&stderr)));
    }

    let wav = std::fs::read(&tmp).map_err(|e| format!("WAV 읽기 실패: {}", e))?;
    let _ = std::fs::remove_file(&tmp);

    if wav.len() < 100 {
        return Err("합성 출력이 비어있습니다.".into());
    }

    Ok(NeuralAudio {
        wav_data: wav,
        sample_rate: 22050,
        engine: format!("coqui-vits-{}", language),
    })
}

/// 커스텀 모델로 합성
pub fn synthesize_custom(text: &str, model_path: &str, config_path: &str) -> Result<NeuralAudio, String> {
    if !tts_available() {
        return Err("Coqui TTS가 설치되어 있지 않습니다.".into());
    }

    let tmp = std::env::temp_dir().join("tts_custom_output.wav");

    let output = Command::new(tts_bin())
        .args([
            "--text", text,
            "--model_path", model_path,
            "--config_path", config_path,
            "--out_path", tmp.to_str().unwrap(),
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("tts 실행 실패: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("tts 오류: {}", stderr.lines().last().unwrap_or(&stderr)));
    }

    let wav = std::fs::read(&tmp).map_err(|e| format!("WAV 읽기 실패: {}", e))?;
    let _ = std::fs::remove_file(&tmp);

    Ok(NeuralAudio {
        wav_data: wav,
        sample_rate: 22050,
        engine: "coqui-custom".into(),
    })
}

/// 학습 데이터 디렉토리 구조 생성
pub fn init_training_dataset(dataset_dir: &str) -> Result<String, String> {
    let base = Path::new(dataset_dir);
    let wavs = base.join("wavs");
    std::fs::create_dir_all(&wavs).map_err(|e| format!("디렉토리 생성 실패: {}", e))?;

    let metadata = base.join("metadata.csv");
    if !metadata.exists() {
        std::fs::write(&metadata, "").map_err(|e| format!("metadata.csv 생성 실패: {}", e))?;
    }

    Ok(format!("학습 데이터 디렉토리 준비 완료: {}\n  wavs/ — WAV 파일 (22050Hz, mono, 16-bit)\n  metadata.csv — 파일명|텍스트|텍스트", dataset_dir))
}

/// 학습 데이터에 항목 추가 (WAV + 텍스트)
pub fn add_training_sample(dataset_dir: &str, wav_data: &[u8], text: &str, sample_id: &str) -> Result<String, String> {
    let base = Path::new(dataset_dir);
    let wav_path = base.join("wavs").join(format!("{}.wav", sample_id));
    std::fs::write(&wav_path, wav_data).map_err(|e| format!("WAV 저장 실패: {}", e))?;

    let metadata = base.join("metadata.csv");
    let line = format!("{}|{}|{}\n", sample_id, text, text);
    let mut content = std::fs::read_to_string(&metadata).unwrap_or_default();
    content.push_str(&line);
    std::fs::write(&metadata, content).map_err(|e| format!("metadata 업데이트 실패: {}", e))?;

    Ok(format!("추가됨: {} → {}", sample_id, text))
}

/// 학습 상태 확인
pub fn get_training_status(dataset_dir: &str) -> serde_json::Value {
    let base = Path::new(dataset_dir);
    let metadata = base.join("metadata.csv");
    let sample_count = if metadata.exists() {
        std::fs::read_to_string(&metadata)
            .unwrap_or_default()
            .lines()
            .filter(|l| !l.trim().is_empty())
            .count()
    } else { 0 };

    // 학습된 모델 확인
    let model_exists = base.join("best_model.pth").exists()
        || base.join("model.pth").exists();

    serde_json::json!({
        "dataset_dir": dataset_dir,
        "sample_count": sample_count,
        "model_trained": model_exists,
        "min_samples": 500,
        "recommended_samples": 2000,
        "ready_to_train": sample_count >= 100,
    })
}

/// 엔진 상태
pub fn check_engines() -> serde_json::Value {
    let coqui_ok = tts_available();
    let models_dir = dirs().join("models");
    let custom_models: Vec<String> = if models_dir.exists() {
        std::fs::read_dir(&models_dir)
            .ok()
            .map(|entries| {
                entries.filter_map(|e| {
                    let e = e.ok()?;
                    if e.path().extension()?.to_str()? == "pth" {
                        Some(e.file_name().to_string_lossy().to_string())
                    } else { None }
                }).collect()
            })
            .unwrap_or_default()
    } else { vec![] };

    serde_json::json!({
        "coqui_tts_installed": coqui_ok,
        "korean_ready": coqui_ok,
        "english_ready": coqui_ok,
        "type": "coqui-vits",
        "pretrained_models": {
            "ko": "tts_models/kor/fairseq/vits",
            "en": "tts_models/en/ljspeech/vits"
        },
        "custom_models": custom_models,
        "install_guide": if coqui_ok { "설치 완료" } else { "bash setup_neural.sh" },
        "training": {
            "supported": true,
            "data_format": "WAV 22050Hz mono 16-bit + metadata.csv",
            "min_samples": 500,
        }
    })
}

fn dirs() -> PathBuf {
    PathBuf::from(".")
}
