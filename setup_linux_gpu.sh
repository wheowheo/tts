#!/bin/bash
# 리눅스 GPU (RTX 8000 / CUDA) 학습 환경 설치 스크립트
set -e

echo "=== TTS 학습 환경 설치 (Linux + CUDA) ==="

# 1. 시스템 확인
echo ""
echo "[1/5] 시스템 확인"
if ! command -v nvidia-smi &>/dev/null; then
    echo "에러: NVIDIA 드라이버가 설치되어 있지 않습니다."
    echo "  sudo apt install nvidia-driver-535 (또는 최신 버전)"
    exit 1
fi
nvidia-smi --query-gpu=name,memory.total,driver_version --format=csv,noheader
echo ""

# 2. 시스템 의존성
echo "[2/5] 시스템 의존성 설치"
sudo apt-get update -qq
sudo apt-get install -y -qq python3.12 python3.12-venv python3.12-dev \
    espeak-ng ffmpeg libsndfile1 git-lfs 2>/dev/null || {
    # python3.12가 없으면 deadsnakes PPA
    sudo apt-get install -y -qq software-properties-common
    sudo add-apt-repository -y ppa:deadsnakes/ppa
    sudo apt-get update -qq
    sudo apt-get install -y -qq python3.12 python3.12-venv python3.12-dev
    sudo apt-get install -y -qq espeak-ng ffmpeg libsndfile1 git-lfs
}

# 3. Python 가상 환경
echo ""
echo "[3/5] Python 가상 환경"
if [ ! -d ".venv" ]; then
    python3.12 -m venv .venv
fi
source .venv/bin/activate
pip install --upgrade pip -q

# 4. PyTorch + CUDA
echo ""
echo "[4/5] PyTorch + CUDA 설치"
pip install torch torchaudio --index-url https://download.pytorch.org/whl/cu124 -q
python -c "
import torch
print(f'PyTorch: {torch.__version__}')
print(f'CUDA: {torch.cuda.is_available()}')
if torch.cuda.is_available():
    print(f'GPU: {torch.cuda.get_device_name(0)}')
    print(f'VRAM: {torch.cuda.get_device_properties(0).total_mem / 1e9:.0f}GB')
"

# 5. Coqui TTS
echo ""
echo "[5/5] Coqui TTS 설치"
pip install "coqui-tts[ko,languages,codec]" -q
pip install "transformers<5" -q

# 학습 디렉토리
mkdir -p training_data/wavs trained_models

# 사전학습 모델 다운로드
echo ""
echo "사전학습 모델 다운로드 (최초 1회)..."
tts --text "테스트" --model_name "tts_models/kor/fairseq/vits" --out_path /tmp/tts_test.wav 2>/dev/null && echo "한국어 모델 OK" || echo "한국어 모델 다운로드 실패 (나중에 다시 시도)"

echo ""
echo "========================================="
echo "  설치 완료!"
echo ""
echo "  합성 테스트:"
echo "    source .venv/bin/activate"
echo "    tts --text '안녕하세요' --model_name 'tts_models/kor/fairseq/vits' --out_path test.wav"
echo ""
echo "  학습 시작:"
echo "    python train_gpu.py --batch_size 48 --epochs 1000"
echo ""
echo "  서버 실행:"
echo "    cargo run -p tts-server --release"
echo "========================================="
