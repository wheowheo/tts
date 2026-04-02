use axum::{
    Router,
    routing::{get, post},
    Json,
    http::StatusCode,
};
use tower_http::services::ServeDir;
use tower_http::cors::CorsLayer;
use tts_core::{
    PipelineInfo, PipelineResult, PipelineStage, SynthesizeRequest, SynthesizeResponse,
    english, hangul, neural, prosody, synthesis,
};

#[derive(serde::Deserialize)]
struct AnalyzeRequest {
    text: String,
}

#[derive(serde::Deserialize)]
struct ProsodyRequest {
    text: String,
    params: Option<prosody::ProsodyParams>,
}

async fn get_pipeline() -> Json<PipelineInfo> {
    Json(PipelineInfo::default_pipeline())
}

async fn analyze(Json(req): Json<AnalyzeRequest>) -> Json<serde_json::Value> {
    let normalized = hangul::normalize_text(&req.text);
    let mut jamo = hangul::decompose_text(&normalized);
    hangul::apply_phonological_rules(&mut jamo);
    let phonemes = hangul::jamo_to_phonemes(&jamo);

    Json(serde_json::json!({
        "original": req.text,
        "normalized": normalized,
        "jamo": jamo,
        "phonemes": phonemes,
    }))
}

async fn generate_prosody(Json(req): Json<ProsodyRequest>) -> Json<serde_json::Value> {
    let normalized = hangul::normalize_text(&req.text);
    let mut jamo = hangul::decompose_text(&normalized);
    hangul::apply_phonological_rules(&mut jamo);
    let phonemes = hangul::jamo_to_phonemes(&jamo);
    let params = req.params.unwrap_or_default();
    let mut prosody_units = prosody::generate_prosody(&phonemes, &params);
    prosody::insert_pauses(&mut prosody_units, &normalized);

    let total_duration: f32 = prosody_units.iter().map(|u| u.duration_ms).sum();

    Json(serde_json::json!({
        "text": req.text,
        "normalized": normalized,
        "prosody": prosody_units,
        "total_duration_ms": total_duration,
        "params": params,
    }))
}

async fn synthesize(Json(req): Json<SynthesizeRequest>) -> Result<Json<SynthesizeResponse>, StatusCode> {
    let lang = req.language.as_deref().unwrap_or("ko");
    let params = req.params.unwrap_or_default();

    let (normalized, phonemes_data, jamo_data, prosody_units) = if lang == "en" {
        let normalized = english::normalize_english(&req.text);
        let en_phonemes = english::english_g2p(&normalized);
        let common_phonemes = english::to_common_phonemes(&en_phonemes);
        let mut prosody_units = prosody::generate_prosody(&common_phonemes, &params);
        prosody::insert_pauses(&mut prosody_units, &normalized);
        (
            normalized,
            serde_json::json!({ "phonemes": &en_phonemes }),
            serde_json::json!({ "note": "영어는 자모 분해를 사용하지 않습니다 (G2P 변환)" }),
            prosody_units,
        )
    } else {
        let normalized = hangul::normalize_text(&req.text);
        let mut jamo = hangul::decompose_text(&normalized);
        hangul::apply_phonological_rules(&mut jamo);
        let phonemes = hangul::jamo_to_phonemes(&jamo);
        let mut prosody_units = prosody::generate_prosody(&phonemes, &params);
        prosody::insert_pauses(&mut prosody_units, &normalized);
        (
            normalized,
            serde_json::json!({ "phonemes": &phonemes }),
            serde_json::json!({ "jamo": &jamo }),
            prosody_units,
        )
    };

    let total_duration: f32 = prosody_units.iter().map(|u| u.duration_ms).sum();

    // 파형 합성
    let pcm_samples = synthesis::synthesize_all(&prosody_units);
    let wav_data = synthesis::encode_wav(&pcm_samples, 44100, 1);
    let audio_base64 = base64_encode(&wav_data);

    let pipeline = vec![
        PipelineResult {
            stage: PipelineStage::TextInput,
            label: "텍스트 입력".into(),
            data: serde_json::json!({ "text": &req.text, "language": lang }),
        },
        PipelineResult {
            stage: PipelineStage::TextNormalization,
            label: "텍스트 정규화".into(),
            data: serde_json::json!({ "normalized": &normalized }),
        },
        PipelineResult {
            stage: PipelineStage::JamoDecomposition,
            label: if lang == "en" { "G2P 변환".into() } else { "자모 분해".into() },
            data: jamo_data,
        },
        PipelineResult {
            stage: PipelineStage::PhonemeConversion,
            label: "음소 변환".into(),
            data: phonemes_data,
        },
        PipelineResult {
            stage: PipelineStage::ProsodyGeneration,
            label: "운율 생성".into(),
            data: serde_json::json!({
                "prosody": &prosody_units,
                "total_duration_ms": total_duration,
                "params": &params,
            }),
        },
        PipelineResult {
            stage: PipelineStage::WaveformSynthesis,
            label: "파형 합성".into(),
            data: serde_json::json!({
                "sample_count": pcm_samples.len(),
                "duration_seconds": pcm_samples.len() as f64 / 44100.0,
            }),
        },
        PipelineResult {
            stage: PipelineStage::PcmOutput,
            label: "PCM 출력".into(),
            data: serde_json::json!({
                "format": "WAV",
                "sample_rate": 44100,
                "channels": 1,
                "bits_per_sample": 16,
                "file_size_bytes": wav_data.len(),
            }),
        },
    ];

    Ok(Json(SynthesizeResponse {
        text: req.text,
        pipeline,
        audio_base64: Some(audio_base64),
        sample_rate: 44100,
        channels: 1,
    }))
}

fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    let chunks = data.chunks(3);
    for chunk in chunks {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let n = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARS[((n >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((n >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARS[((n >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARS[(n & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}

// === 신경망 TTS API ===

#[derive(serde::Deserialize)]
struct NeuralRequest {
    text: String,
    language: Option<String>,
    model_path: Option<String>,
    config_path: Option<String>,
}

async fn neural_synthesize(Json(req): Json<NeuralRequest>) -> Json<serde_json::Value> {
    let lang = req.language.as_deref().unwrap_or("ko");

    let result = if let (Some(mp), Some(cp)) = (&req.model_path, &req.config_path) {
        neural::synthesize_custom(&req.text, mp, cp)
    } else {
        neural::synthesize(&req.text, lang)
    };

    match result {
        Ok(audio) => {
            let b64 = base64_encode(&audio.wav_data);
            Json(serde_json::json!({
                "audio_base64": b64, "sample_rate": audio.sample_rate,
                "engine": audio.engine, "format": "wav", "text": req.text,
            }))
        }
        Err(e) => Json(serde_json::json!({ "error": e, "text": req.text })),
    }
}

async fn get_engines() -> Json<serde_json::Value> {
    Json(neural::check_engines())
}

#[derive(serde::Deserialize)]
struct TrainingDataReq {
    action: String, // "init", "status", "add"
    dataset_dir: Option<String>,
    text: Option<String>,
    sample_id: Option<String>,
    wav_base64: Option<String>,
}

async fn training_api(Json(req): Json<TrainingDataReq>) -> Json<serde_json::Value> {
    let dir = req.dataset_dir.as_deref().unwrap_or("training_data");

    match req.action.as_str() {
        "init" => {
            match neural::init_training_dataset(dir) {
                Ok(msg) => Json(serde_json::json!({ "ok": true, "message": msg })),
                Err(e) => Json(serde_json::json!({ "ok": false, "error": e })),
            }
        }
        "status" => Json(neural::get_training_status(dir)),
        "add" => {
            let text = req.text.as_deref().unwrap_or("");
            let id = req.sample_id.as_deref().unwrap_or("sample_0000");
            if let Some(b64) = &req.wav_base64 {
                // base64 디코딩
                let wav = base64_decode(b64);
                match neural::add_training_sample(dir, &wav, text, id) {
                    Ok(msg) => Json(serde_json::json!({ "ok": true, "message": msg })),
                    Err(e) => Json(serde_json::json!({ "ok": false, "error": e })),
                }
            } else {
                Json(serde_json::json!({ "ok": false, "error": "wav_base64 필요" }))
            }
        }
        _ => Json(serde_json::json!({ "ok": false, "error": "알 수 없는 action" })),
    }
}

fn base64_decode(data: &str) -> Vec<u8> {
    let mut result = Vec::new();
    let chars: Vec<u8> = data.bytes().filter(|&b| b != b'\n' && b != b'\r' && b != b'=').collect();
    let lookup = |c: u8| -> u8 {
        match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' => 62, b'/' => 63, _ => 0,
        }
    };
    for chunk in chars.chunks(4) {
        let n = chunk.iter().enumerate().fold(0u32, |acc, (i, &c)| acc | ((lookup(c) as u32) << (18 - i * 6)));
        result.push((n >> 16) as u8);
        if chunk.len() > 2 { result.push((n >> 8) as u8); }
        if chunk.len() > 3 { result.push(n as u8); }
    }
    result
}

#[tokio::main]
async fn main() {
    let api_routes = Router::new()
        .route("/pipeline", get(get_pipeline))
        .route("/analyze", post(analyze))
        .route("/prosody", post(generate_prosody))
        .route("/synthesize", post(synthesize))
        .route("/neural-synthesize", post(neural_synthesize))
        .route("/engines", get(get_engines))
        .route("/training", post(training_api));

    let app = Router::new()
        .nest("/api", api_routes)
        .fallback_service(ServeDir::new("static"))
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("🔊 TTS Server running at http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}
