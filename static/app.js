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

        // 단계별 맞춤 렌더링
        let content = '';
        if (step.stage === 'JamoDecomposition' && step.data.jamo) {
            content = renderJamoVisual(step.data.jamo);
        } else if (step.stage === 'PhonemeConversion' && step.data.phonemes) {
            content = renderPhonemeVisual(step.data.phonemes);
        } else {
            content = `<div class="result-data">${JSON.stringify(step.data, null, 2)}</div>`;
        }

        card.innerHTML = `<div class="result-label">${step.label}</div>${content}`;
        container.appendChild(card);
    });

    const audioSection = document.getElementById('audio-section');
    if (data.audio_base64) {
        audioSection.style.display = 'block';
    } else {
        audioSection.style.display = 'block';
        drawEmptyWaveform();
    }
}

// 자모 분해 시각화: 각 글자를 초성/중성/종성으로 분리하여 표시
function renderJamoVisual(jamoList) {
    if (!jamoList || jamoList.length === 0) {
        return '<div class="result-data">분해할 한글 문자가 없습니다.</div>';
    }

    let html = '<div class="jamo-visual">';
    jamoList.forEach(j => {
        html += `
            <div class="jamo-card">
                <div class="jamo-char">${j.character}</div>
                <div class="jamo-parts">
                    <span class="jamo-cho" title="초성">${j.choseong}</span>
                    <span class="jamo-jung" title="중성">${j.jungseong}</span>
                    <span class="jamo-jong" title="종성">${j.jongseong || '·'}</span>
                </div>
                <div class="jamo-labels">
                    <span>초</span><span>중</span><span>종</span>
                </div>
            </div>
        `;
    });
    html += '</div>';
    return html;
}

// 음소 시각화: 음소 시퀀스를 색상으로 구분하여 표시
function renderPhonemeVisual(phonemes) {
    if (!phonemes || phonemes.length === 0) {
        return '<div class="result-data">음소가 없습니다.</div>';
    }

    let html = '<div class="phoneme-visual">';
    phonemes.forEach(p => {
        const typeClass = p.phoneme_type === 'Consonant' ? 'ph-consonant'
            : p.phoneme_type === 'Vowel' ? 'ph-vowel'
            : 'ph-other';
        html += `
            <div class="phoneme-chip ${typeClass}" title="${p.source_char}">
                <span class="ph-symbol">${p.symbol}</span>
                <span class="ph-type">${p.phoneme_type === 'Consonant' ? '자음' : '모음'}</span>
            </div>
        `;
    });
    html += '</div>';

    // 음소 시퀀스 텍스트
    const sequence = phonemes.map(p => p.symbol).join(' ');
    html += `<div class="phoneme-sequence">${sequence}</div>`;

    return html;
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

    ctx.strokeStyle = '#333';
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.moveTo(0, h / 2);
    ctx.lineTo(w, h / 2);
    ctx.stroke();

    ctx.fillStyle = '#999';
    ctx.font = '14px sans-serif';
    ctx.textAlign = 'center';
    ctx.fillText('Phase 4에서 파형이 표시됩니다', w / 2, h / 2 - 10);
}

document.addEventListener('DOMContentLoaded', loadPipeline);
