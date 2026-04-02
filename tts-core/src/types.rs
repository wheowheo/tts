use serde::{Deserialize, Serialize};

/// TTS 파이프라인의 각 단계
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PipelineStage {
    TextInput,
    TextNormalization,
    JamoDecomposition,
    PhonemeConversion,
    ProsodyGeneration,
    WaveformSynthesis,
    PcmOutput,
}

/// 파이프라인 단계별 결과
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineResult {
    pub stage: PipelineStage,
    pub label: String,
    pub data: serde_json::Value,
}

/// 합성 요청
#[derive(Debug, Deserialize)]
pub struct SynthesizeRequest {
    pub text: String,
    pub language: Option<String>,
}

/// 합성 응답 (파이프라인 전체 결과 포함)
#[derive(Debug, Serialize)]
pub struct SynthesizeResponse {
    pub text: String,
    pub pipeline: Vec<PipelineResult>,
    pub audio_base64: Option<String>,
    pub sample_rate: u32,
    pub channels: u16,
}

/// 파이프라인 상태 정보
#[derive(Debug, Serialize)]
pub struct PipelineInfo {
    pub stages: Vec<StageInfo>,
}

#[derive(Debug, Serialize)]
pub struct StageInfo {
    pub name: String,
    pub description: String,
    pub order: u32,
}

impl PipelineInfo {
    pub fn default_pipeline() -> Self {
        PipelineInfo {
            stages: vec![
                StageInfo {
                    name: "text_input".into(),
                    description: "원본 텍스트 입력".into(),
                    order: 0,
                },
                StageInfo {
                    name: "text_normalization".into(),
                    description: "텍스트 정규화 (숫자, 기호 → 한글)".into(),
                    order: 1,
                },
                StageInfo {
                    name: "jamo_decomposition".into(),
                    description: "한글 자모 분해 (초성/중성/종성)".into(),
                    order: 2,
                },
                StageInfo {
                    name: "phoneme_conversion".into(),
                    description: "음소(Phoneme) 변환".into(),
                    order: 3,
                },
                StageInfo {
                    name: "prosody_generation".into(),
                    description: "운율 생성 (피치, 길이, 강세)".into(),
                    order: 4,
                },
                StageInfo {
                    name: "waveform_synthesis".into(),
                    description: "파형 합성 (Formant Synthesis)".into(),
                    order: 5,
                },
                StageInfo {
                    name: "pcm_output".into(),
                    description: "PCM 오디오 출력".into(),
                    order: 6,
                },
            ],
        }
    }
}
