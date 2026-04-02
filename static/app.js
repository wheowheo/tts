const API_BASE = '';

async function loadPipeline() {
    try {
        const res = await fetch(`${API_BASE}/api/pipeline`);
        const info = await res.json();
        renderPipelineStages(info.stages);
    } catch (e) {
        console.error('파이프라인 로드 실패:', e);
    }
}

function renderPipelineStages(stages) {
    const container = document.getElementById('pipeline-stages');
    container.innerHTML = '';
    stages.forEach((stage, i) => {
        if (i > 0) {
            const arrow = document.createElement('span');
            arrow.className = 'stage-arrow';
            arrow.textContent = '→';
            container.appendChild(arrow);
        }
        const box = document.createElement('div');
        box.className = 'stage-box';
        box.id = `stage-${stage.name}`;
        box.innerHTML = `
            <div class="stage-name">${stage.description}</div>
            <div class="stage-desc">#${stage.order}</div>
        `;
        container.appendChild(box);
    });
}

async function synthesize() {
    const text = document.getElementById('text-input').value.trim();
    if (!text) {
        alert('텍스트를 입력하세요.');
        return;
    }

    const lang = document.getElementById('language-select').value;
    const btn = document.getElementById('synthesize-btn');
    btn.disabled = true;
    btn.textContent = '처리 중...';

    // 모든 stage를 active로 초기화
    document.querySelectorAll('.stage-box').forEach(b => {
        b.classList.remove('active', 'completed');
    });

    try {
        const res = await fetch(`${API_BASE}/api/synthesize`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ text, language: lang }),
        });

        const data = await res.json();
        renderResults(data);

        // 애니메이션: 단계별로 순차 활성화
        animateStages(data.pipeline);

    } catch (e) {
        console.error('합성 실패:', e);
        alert('합성 요청에 실패했습니다.');
    } finally {
        btn.disabled = false;
        btn.textContent = '합성 실행';
    }
}

function animateStages(pipeline) {
    const stageNames = [
        'text_input', 'text_normalization', 'jamo_decomposition',
        'phoneme_conversion', 'prosody_generation', 'waveform_synthesis', 'pcm_output'
    ];

    stageNames.forEach((name, i) => {
        setTimeout(() => {
            const el = document.getElementById(`stage-${name}`);
            if (el) {
                el.classList.add('active');
                setTimeout(() => {
                    el.classList.remove('active');
                    el.classList.add('completed');
                }, 300);
            }
        }, i * 200);
    });
}

function renderResults(data) {
    const section = document.getElementById('result-section');
    const container = document.getElementById('pipeline-results');
    section.style.display = 'block';
    container.innerHTML = '';

    data.pipeline.forEach(step => {
        const card = document.createElement('div');
        card.className = 'result-card';
        card.innerHTML = `
            <div class="result-label">${step.label}</div>
            <div class="result-data">${JSON.stringify(step.data, null, 2)}</div>
        `;
        container.appendChild(card);
    });

    // 오디오 섹션 표시 (Phase 4에서 활성화)
    const audioSection = document.getElementById('audio-section');
    if (data.audio_base64) {
        audioSection.style.display = 'block';
        // Phase 4에서 구현
    } else {
        audioSection.style.display = 'block';
        drawEmptyWaveform();
    }
}

function drawEmptyWaveform() {
    const canvas = document.getElementById('waveform-canvas');
    const ctx = canvas.getContext('2d');
    canvas.width = canvas.clientWidth * window.devicePixelRatio;
    canvas.height = canvas.clientHeight * window.devicePixelRatio;
    ctx.scale(window.devicePixelRatio, window.devicePixelRatio);

    const w = canvas.clientWidth;
    const h = canvas.clientHeight;

    ctx.fillStyle = '#16213e';
    ctx.fillRect(0, 0, w, h);

    // 중앙선
    ctx.strokeStyle = '#333';
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.moveTo(0, h / 2);
    ctx.lineTo(w, h / 2);
    ctx.stroke();

    // "Phase 4에서 파형이 표시됩니다" 텍스트
    ctx.fillStyle = '#999';
    ctx.font = '14px sans-serif';
    ctx.textAlign = 'center';
    ctx.fillText('Phase 4에서 파형이 표시됩니다', w / 2, h / 2 - 10);
}

// 페이지 로드 시 파이프라인 정보 가져오기
document.addEventListener('DOMContentLoaded', loadPipeline);
