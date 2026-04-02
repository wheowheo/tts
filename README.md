# TTS Generator Editor

텍스트를 소리로 바꾸는 과정을 시각화하고, 신경망 TTS 모델을 학습할 수 있는 도구.

## 빠른 시작 (리눅스 RTX 8000)

```bash
git clone https://github.com/wheowheo/tts.git
cd tts
git checkout feature/trainable-tts

# 1. 전체 설치 (Rust + Python + CUDA + Coqui TTS)
bash deploy_linux.sh

# 2. 서버 실행
./target/release/tts-server
# http://서버IP:3000 접속
```

## 신경망 TTS 합성 (사전학습 모델)

설치 완료 후 바로 사용 가능:

```bash
source .venv/bin/activate

# 한국어
tts --text "안녕하세요 봉봉 테스트입니다" \
    --model_name "tts_models/kor/fairseq/vits" \
    --out_path output_ko.wav

# 영어
tts --text "Hello world" \
    --model_name "tts_models/en/ljspeech/vits" \
    --out_path output_en.wav

# 재생
aplay output_ko.wav
```

웹 UI에서도 동일하게 사용 가능:
1. `http://서버IP:3000` 접속
2. 엔진: **신경망 VITS** 선택
3. 텍스트 입력 → **합성** 클릭 → **재생**

## 커스텀 음성 학습 (GPU)

나만의 목소리로 TTS 모델을 학습합니다.

### 1단계: 학습 데이터 준비

```bash
mkdir -p training_data/wavs
```

`training_data/wavs/`에 WAV 파일 배치:
- 포맷: **22050Hz, mono, 16-bit PCM**
- 길이: 각 파일 **2~10초**
- 내용: 깨끗한 음성, 배경 소음 없이
- 수량: 최소 50개(테스트), 권장 **500개 이상** (30분+)

`training_data/metadata.csv` 작성:
```
audio_0001|안녕하세요 반갑습니다|안녕하세요 반갑습니다
audio_0002|오늘 날씨가 좋습니다|오늘 날씨가 좋습니다
audio_0003|텍스트를 소리로 바꾸는 테스트|텍스트를 소리로 바꾸는 테스트
```
형식: `파일명(확장자 없이)|원본 텍스트|정규화 텍스트`

### WAV 변환 (기존 녹음 파일이 있을 때)

```bash
# MP3/M4A → WAV 변환
ffmpeg -i input.mp3 -ar 22050 -ac 1 -sample_fmt s16 training_data/wavs/audio_0001.wav

# 긴 파일을 10초 단위로 분할
ffmpeg -i long_recording.wav -f segment -segment_time 10 \
    -ar 22050 -ac 1 training_data/wavs/clip_%04d.wav
```

### 2단계: GPU 학습

```bash
source .venv/bin/activate

# RTX 8000 권장 설정
python train_gpu.py \
    --dataset_dir training_data \
    --batch_size 48 \
    --epochs 1000 \
    --fp16

# 학습 중간에 끊겼을 때 재개
python train_gpu.py \
    --dataset_dir training_data \
    --batch_size 48 \
    --epochs 1000 \
    --resume trained_models/custom_korean_vits-*/checkpoint_*.pth
```

GPU별 권장 batch_size:

| GPU | VRAM | batch_size |
|-----|------|-----------|
| RTX 8000 | 48GB | **48~64** |
| RTX 4090 | 24GB | 24~32 |
| RTX 3090 | 24GB | 24~32 |
| T4 | 16GB | 12~16 |

학습 예상 시간 (2000샘플, 1000 epochs):

| GPU | 시간 |
|-----|------|
| RTX 8000 | **8~15시간** |
| RTX 4090 | 5~10시간 |
| T4 | 24~48시간 |
| CPU | 2~4주 |

### 3단계: 학습된 모델 사용

```bash
# CLI로 합성
tts --text "안녕하세요" \
    --model_path trained_models/best_model.pth \
    --config_path trained_models/config.json \
    --out_path test.wav

aplay test.wav
```

웹 UI에서 사용하려면 서버 재시작 후 커스텀 모델 경로를 지정합니다.

## 웹 UI 사용법

`http://서버IP:3000`

### Step 1: 합성
- 텍스트 입력
- 언어 선택 (한국어/English)
- 엔진 선택:
  - **포먼트 합성** — 설치 없이 동작, 학습용 (음질 낮음)
  - **신경망 VITS** — Coqui TTS, 고품질
- [합성] 클릭

### Step 2: 듣기
- [재생] → 소리 확인
- [WAV 저장] → 파일 다운로드
- 품질 요약 자동 표시 (무음 비율, 클릭 수)

### Step 3: 조절 (포먼트 전용)
- 목소리 높낮이 / 속도 / 크기 슬라이더
- 프리셋: 기본값, 높은 목소리, 낮은 목소리, 빠르게, 느리게

### 상세 분석
- 파이프라인: 텍스트→자모→음소→운율→합성 과정
- 스펙트로그램: FFT 주파수 분석
- 품질 검증: 클릭, 잡음, 동적 범위

### 학습 데이터 관리
- [학습 폴더 초기화] → training_data/ 생성
- [상태 확인] → 샘플 수, 학습 가능 여부

## API

| 메서드 | 경로 | 설명 |
|--------|------|------|
| GET | `/api/pipeline` | 파이프라인 단계 정보 |
| GET | `/api/engines` | 엔진 상태 (Coqui 설치 여부, 모델 목록) |
| POST | `/api/synthesize` | 포먼트 합성 |
| POST | `/api/neural-synthesize` | 신경망 합성 |
| POST | `/api/analyze` | 텍스트 분석 (자모 분해 + 음운 규칙) |
| POST | `/api/prosody` | 운율 생성 |
| POST | `/api/training` | 학습 데이터 관리 (init/status/add) |

### 신경망 합성 예시
```bash
curl -X POST http://localhost:3000/api/neural-synthesize \
  -H 'Content-Type: application/json' \
  -d '{"text":"안녕하세요","language":"ko"}' \
  | jq -r '.audio_base64' | base64 -d > output.wav
```

## 브랜치

| 브랜치 | 용도 |
|--------|------|
| `main` | 포먼트 합성 (설치 없이 `cargo run -p tts-server`) |
| `feature/trainable-tts` | 신경망 VITS + GPU 학습 |
| `feature/macos-tts` | macOS say 연동 |

## 프로젝트 구조

```
tts/
├── tts-core/                # 핵심 라이브러리
│   └── src/
│       ├── hangul.rs        # 한글 자모 분해 + 음운 규칙
│       ├── english.rs       # 영어 G2P
│       ├── prosody.rs       # 운율 생성
│       ├── synthesis.rs     # 포먼트 합성 (Klatt)
│       ├── neural.rs        # Coqui TTS 연동
│       └── types.rs         # 공통 타입
├── tts-server/              # Axum REST 서버
├── static/                  # 웹 UI
│   ├── index.html
│   ├── app.js
│   └── style.css
├── setup_linux_gpu.sh       # 리눅스 GPU 환경 설치
├── deploy_linux.sh          # 리눅스 배포 (Rust + Python + systemd)
├── setup_neural.sh          # macOS 설치
├── train_gpu.py             # GPU 학습 스크립트
├── train.py                 # CPU 학습 (폴백)
├── PLAN.md                  # 개발 계획
├── TUNING_PLAN.md           # 튜닝 계획
├── MANUAL.md                # 사용자 매뉴얼
└── 로봇소리 탈출기.md        # 튜닝 여정 기록
```

## 라이선스

학습용 프로젝트
