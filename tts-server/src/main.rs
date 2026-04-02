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
    hangul,
};

#[derive(serde::Deserialize)]
struct AnalyzeRequest {
    text: String,
}

async fn get_pipeline() -> Json<PipelineInfo> {
    Json(PipelineInfo::default_pipeline())
}

async fn analyze(Json(req): Json<AnalyzeRequest>) -> Json<serde_json::Value> {
    let normalized = hangul::normalize_text(&req.text);
    let jamo = hangul::decompose_text(&normalized);
    let phonemes = hangul::jamo_to_phonemes(&jamo);

    Json(serde_json::json!({
        "original": req.text,
        "normalized": normalized,
        "jamo": jamo,
        "phonemes": phonemes,
    }))
}

async fn synthesize(Json(req): Json<SynthesizeRequest>) -> Result<Json<SynthesizeResponse>, StatusCode> {
    let normalized = hangul::normalize_text(&req.text);
    let jamo = hangul::decompose_text(&normalized);
    let phonemes = hangul::jamo_to_phonemes(&jamo);

    let pipeline = vec![
        PipelineResult {
            stage: PipelineStage::TextInput,
            label: "텍스트 입력".into(),
            data: serde_json::json!({ "text": &req.text }),
        },
        PipelineResult {
            stage: PipelineStage::TextNormalization,
            label: "텍스트 정규화".into(),
            data: serde_json::json!({ "normalized": &normalized }),
        },
        PipelineResult {
            stage: PipelineStage::JamoDecomposition,
            label: "자모 분해".into(),
            data: serde_json::json!({ "jamo": &jamo }),
        },
        PipelineResult {
            stage: PipelineStage::PhonemeConversion,
            label: "음소 변환".into(),
            data: serde_json::json!({ "phonemes": &phonemes }),
        },
        PipelineResult {
            stage: PipelineStage::ProsodyGeneration,
            label: "운율 생성".into(),
            data: serde_json::json!({ "prosody": [], "note": "Phase 3에서 구현 예정" }),
        },
        PipelineResult {
            stage: PipelineStage::WaveformSynthesis,
            label: "파형 합성".into(),
            data: serde_json::json!({ "note": "Phase 4에서 구현 예정" }),
        },
        PipelineResult {
            stage: PipelineStage::PcmOutput,
            label: "PCM 출력".into(),
            data: serde_json::json!({ "note": "Phase 4에서 구현 예정" }),
        },
    ];

    Ok(Json(SynthesizeResponse {
        text: req.text,
        pipeline,
        audio_base64: None,
        sample_rate: 44100,
        channels: 1,
    }))
}

#[tokio::main]
async fn main() {
    let api_routes = Router::new()
        .route("/pipeline", get(get_pipeline))
        .route("/analyze", post(analyze))
        .route("/synthesize", post(synthesize));

    let app = Router::new()
        .nest("/api", api_routes)
        .fallback_service(ServeDir::new("static"))
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("🔊 TTS Server running at http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}
