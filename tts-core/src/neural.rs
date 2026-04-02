/// 로컬 TTS 엔진 모듈
/// macOS `say` 명령 기반 — 설치 불필요, 인터넷 불필요

use std::process::{Command, Stdio};

pub struct NeuralAudio {
    pub wav_data: Vec<u8>,
    pub sample_rate: u32,
    pub engine: String,
}

/// 실제 작동하는 음성만 반환 (say로 테스트 합성하여 빈 출력 필터링)
fn get_working_voices(lang_prefix: &str) -> Vec<(String, String)> {
    let output = Command::new("say").args(["-v", "?"]).output().ok();
    let Some(out) = output else { return vec![] };
    let list = String::from_utf8_lossy(&out.stdout);

    let test_text = if lang_prefix.starts_with("ko") { "테스트" } else { "test" };

    list.lines()
        .filter(|l| l.contains(lang_prefix))
        .filter_map(|line| {
            let name = line.split_whitespace().next()?.to_string();
            // 실제 합성 테스트 — AIFF가 10KB 이상이면 작동
            let tmp = std::env::temp_dir().join(format!("voice_check_{}.aiff", &name));
            let ok = Command::new("say")
                .args(["-v", &name, "-o", tmp.to_str()?, test_text])
                .stdout(Stdio::null()).stderr(Stdio::null())
                .status().map(|s| s.success()).unwrap_or(false);
            let size = std::fs::metadata(&tmp).map(|m| m.len()).unwrap_or(0);
            let _ = std::fs::remove_file(&tmp);

            if ok && size > 10000 {
                let desc = if let Some(start) = line.find('(') {
                    if let Some(end) = line.find(')') {
                        format!("{} ({})", &name, &line[start+1..end])
                    } else { name.clone() }
                } else { name.clone() };
                Some((name, desc))
            } else {
                None // 다운로드 안 된 음성 제외
            }
        })
        .collect()
}

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
    // 음성 존재 확인
    let voice_list = Command::new("say").args(["-v", "?"]).output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();
    if !voice_list.lines().any(|l| l.starts_with(voice) || l.contains(&format!("{} ", voice))) {
        return Err(format!("음성 '{}'를 찾을 수 없습니다. 사용 가능: Yuna, Shelley, Sandy, Reed, Eddy, Flo (한국어), Samantha, Alex, Daniel, Karen (영어)", voice));
    }

    let tmp_aiff = std::env::temp_dir().join("tts_say_output.aiff");
    let tmp_wav = std::env::temp_dir().join("tts_say_output.wav");

    // say → AIFF
    let status = Command::new("say")
        .args(["-v", voice, "-o", tmp_aiff.to_str().unwrap(), text])
        .status()
        .map_err(|e| format!("say 실행 실패: {}", e))?;

    if !status.success() {
        let _ = std::fs::remove_file(&tmp_aiff);
        return Err(format!("say 오류 — 음성 '{}'를 찾을 수 없습니다", voice));
    }

    // AIFF 파일 크기 확인 (10KB 미만이면 음성 데이터 없음)
    let aiff_size = std::fs::metadata(&tmp_aiff).map(|m| m.len()).unwrap_or(0);
    if aiff_size < 10000 {
        let _ = std::fs::remove_file(&tmp_aiff);
        return Err(format!("음성 '{}'의 데이터가 다운로드되지 않았습니다. macOS 설정 > 손쉬운 사용 > 음성 콘텐츠에서 다운로드하세요.", voice));
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

    let ko_voices = if say_ok { get_working_voices("ko_KR") } else { vec![] };
    let en_voices = if say_ok { get_working_voices("en_US") } else { vec![] };

    serde_json::json!({
        "say_available": say_ok,
        "ffmpeg_installed": ffmpeg_ok,
        "korean_ready": say_ok && ffmpeg_ok && !ko_voices.is_empty(),
        "english_ready": say_ok && ffmpeg_ok && !en_voices.is_empty(),
        "korean_voices": ko_voices.iter().map(|(n, d)| serde_json::json!({"id": n, "name": d})).collect::<Vec<_>>(),
        "english_voices": en_voices.iter().map(|(n, d)| serde_json::json!({"id": n, "name": d})).collect::<Vec<_>>(),
        "type": "local",
        "install_guide": if ffmpeg_ok { "설치 완료" } else { "brew install ffmpeg" },
    })
}
