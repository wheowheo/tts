#!/bin/bash
# 신경망 TTS (Coqui VITS) 설치 스크립트
set -e

echo "=== 신경망 TTS 설치 ==="

# 1. 의존성 확인
if ! command -v python3.12 &>/dev/null; then
    echo "Python 3.12 설치 중..."
    brew install python@3.12
fi

if ! command -v espeak-ng &>/dev/null; then
    echo "espeak-ng 설치 중..."
    brew install espeak-ng
fi

# 2. 가상 환경
if [ ! -d ".venv" ]; then
    echo "Python 가상 환경 생성..."
    python3.12 -m venv .venv
fi

source .venv/bin/activate

# 3. Coqui TTS 설치
echo "Coqui TTS 설치 중..."
pip install --upgrade pip
pip install torch torchaudio
pip install "coqui-tts[ko,languages,codec]"
pip install "transformers<5"

# 4. 학습 데이터 디렉토리
mkdir -p training_data/wavs

# 5. 한국어 사전학습 모델 사전 다운로드
echo "한국어 모델 사전 다운로드..."
tts --text "테스트" --model_name "tts_models/kor/fairseq/vits" --out_path /dev/null 2>/dev/null || true

echo ""
echo "=== 설치 완료 ==="
echo "서버 실행: cargo run -p tts-server"
echo "합성 테스트: source .venv/bin/activate && tts --text '안녕하세요' --model_name 'tts_models/kor/fairseq/vits' --out_path test.wav"
