#!/bin/bash
# 리눅스 서버 배포 스크립트
# 사용법: scp -r . user@server:/opt/tts && ssh user@server 'cd /opt/tts && bash deploy_linux.sh'
set -e

echo "=== TTS 서버 배포 (Linux) ==="

# 1. Rust 설치 (없으면)
if ! command -v cargo &>/dev/null; then
    echo "Rust 설치..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi
echo "Rust: $(rustc --version)"

# 2. GPU 학습 환경 설치
bash setup_linux_gpu.sh

# 3. Rust 서버 빌드 (릴리스)
echo ""
echo "서버 빌드..."
cargo build --release -p tts-server

# 4. systemd 서비스 파일 생성
DEPLOY_DIR=$(pwd)
cat > /tmp/tts-server.service << SVCEOF
[Unit]
Description=TTS Generator Server
After=network.target

[Service]
Type=simple
WorkingDirectory=${DEPLOY_DIR}
ExecStart=${DEPLOY_DIR}/target/release/tts-server
Restart=always
RestartSec=5
Environment=PATH=${DEPLOY_DIR}/.venv/bin:/usr/local/bin:/usr/bin:/bin

[Install]
WantedBy=multi-user.target
SVCEOF

echo ""
echo "========================================="
echo "  배포 완료!"
echo ""
echo "  수동 실행:"
echo "    cd ${DEPLOY_DIR}"
echo "    ./target/release/tts-server"
echo ""
echo "  systemd 서비스 등록:"
echo "    sudo cp /tmp/tts-server.service /etc/systemd/system/"
echo "    sudo systemctl daemon-reload"
echo "    sudo systemctl enable --now tts-server"
echo ""
echo "  접속: http://$(hostname -I | awk '{print $1}'):3000"
echo ""
echo "  GPU 학습:"
echo "    source .venv/bin/activate"
echo "    python train_gpu.py --batch_size 48 --epochs 1000"
echo "========================================="
