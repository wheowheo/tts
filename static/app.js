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

        let content = '';
        if (step.stage === 'JamoDecomposition' && step.data.jamo) {
            content = renderJamoVisual(step.data.jamo);
        } else if (step.stage === 'PhonemeConversion' && step.data.phonemes) {
            content = renderPhonemeVisual(step.data.phonemes);
        } else if (step.stage === 'ProsodyGeneration' && step.data.prosody) {
            content = renderProsodyVisual(step.data);
        } else {
            content = `<div class="result-data">${JSON.stringify(step.data, null, 2)}</div>`;
        }

        card.innerHTML = `<div class="result-label">${step.label}</div>${content}`;
        container.appendChild(card);
    });

    const audioSection = document.getElementById('audio-section');
    audioSection.style.display = 'block';
    if (data.audio_base64) {
        setupAudioPlayback(data.audio_base64);
    } else {
        drawEmptyWaveform();
    }
}

// 자모 분해 시각화
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

// 음소 시각화
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

    const sequence = phonemes.map(p => p.symbol).join(' ');
    html += `<div class="phoneme-sequence">${sequence}</div>`;
    return html;
}

// 운율 시각화: 피치 곡선 + 음소별 길이 바 차트
function renderProsodyVisual(prosodyData) {
    const units = prosodyData.prosody;
    if (!units || units.length === 0) {
        return '<div class="result-data">운율 데이터가 없습니다.</div>';
    }

    const totalDuration = prosodyData.total_duration_ms || 0;

    let html = '';

    // 요약 정보
    html += `<div class="prosody-info">총 길이: ${totalDuration.toFixed(0)}ms | 기본 피치: ${prosodyData.params?.base_pitch_hz || 150}Hz | 속도: ${prosodyData.params?.speed_factor || 1.0}x</div>`;

    // 피치 곡선 캔버스
    html += '<div class="prosody-chart-label">피치 곡선 (Hz)</div>';
    html += '<canvas id="pitch-canvas" class="prosody-canvas" width="800" height="120"></canvas>';

    // 음소별 길이 바 차트
    html += '<div class="prosody-chart-label">음소별 길이 (ms)</div>';
    html += '<div class="duration-bars">';
    units.forEach(u => {
        const maxDur = 200;
        const pct = Math.min(u.duration_ms / maxDur * 100, 100);
        const color = u.phoneme_type === 'Vowel' ? '#4ade80'
            : u.phoneme_type === 'Consonant' ? '#60a5fa'
            : '#fbbf24';
        html += `
            <div class="dur-bar-item">
                <span class="dur-label">${u.phoneme}</span>
                <div class="dur-bar-track">
                    <div class="dur-bar-fill" style="width:${pct}%;background:${color}"></div>
                </div>
                <span class="dur-value">${u.duration_ms.toFixed(0)}</span>
            </div>
        `;
    });
    html += '</div>';

    // 피치 캔버스는 DOM에 추가된 후 그려야 하므로 setTimeout 사용
    setTimeout(() => drawPitchCurve(units), 50);

    return html;
}

function drawPitchCurve(units) {
    const canvas = document.getElementById('pitch-canvas');
    if (!canvas) return;

    const dpr = window.devicePixelRatio || 1;
    const w = canvas.clientWidth;
    const h = canvas.clientHeight;
    canvas.width = w * dpr;
    canvas.height = h * dpr;
    const ctx = canvas.getContext('2d');
    ctx.scale(dpr, dpr);

    ctx.fillStyle = '#16213e';
    ctx.fillRect(0, 0, w, h);

    // 피치 값이 있는 유닛만 필터
    const pitched = units.filter(u => u.pitch_hz > 0);
    if (pitched.length === 0) return;

    const pitches = pitched.map(u => u.pitch_hz);
    const minP = Math.min(...pitches) - 10;
    const maxP = Math.max(...pitches) + 10;
    const rangeP = maxP - minP || 1;

    // 그리드
    ctx.strokeStyle = '#333';
    ctx.lineWidth = 0.5;
    for (let i = 0; i <= 4; i++) {
        const y = h * 0.1 + (h * 0.8) * (i / 4);
        ctx.beginPath();
        ctx.moveTo(0, y);
        ctx.lineTo(w, y);
        ctx.stroke();
        const val = maxP - (rangeP * i / 4);
        ctx.fillStyle = '#666';
        ctx.font = '10px sans-serif';
        ctx.textAlign = 'left';
        ctx.fillText(`${val.toFixed(0)}`, 4, y - 3);
    }

    // 피치 곡선
    ctx.strokeStyle = '#e94560';
    ctx.lineWidth = 2;
    ctx.beginPath();
    pitched.forEach((u, i) => {
        const x = (i / (pitched.length - 1 || 1)) * (w - 40) + 20;
        const y = h * 0.1 + (1 - (u.pitch_hz - minP) / rangeP) * (h * 0.8);
        if (i === 0) ctx.moveTo(x, y);
        else ctx.lineTo(x, y);
    });
    ctx.stroke();

    // 포인트 + 레이블
    pitched.forEach((u, i) => {
        const x = (i / (pitched.length - 1 || 1)) * (w - 40) + 20;
        const y = h * 0.1 + (1 - (u.pitch_hz - minP) / rangeP) * (h * 0.8);

        ctx.fillStyle = '#e94560';
        ctx.beginPath();
        ctx.arc(x, y, 3, 0, Math.PI * 2);
        ctx.fill();

        ctx.fillStyle = '#ccc';
        ctx.font = '9px sans-serif';
        ctx.textAlign = 'center';
        ctx.fillText(u.phoneme, x, h - 4);
    });
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
    ctx.fillText('합성 실행을 눌러 파형을 생성하세요', w / 2, h / 2 - 10);
}

// 오디오 재생 및 파형 시각화
let currentAudioBuffer = null;
let audioContext = null;
let currentSource = null;

function setupAudioPlayback(base64Audio) {
    const binaryStr = atob(base64Audio);
    const bytes = new Uint8Array(binaryStr.length);
    for (let i = 0; i < binaryStr.length; i++) {
        bytes[i] = binaryStr.charCodeAt(i);
    }

    if (!audioContext) {
        audioContext = new (window.AudioContext || window.webkitAudioContext)();
    }

    audioContext.decodeAudioData(bytes.buffer.slice(0)).then(buffer => {
        currentAudioBuffer = buffer;
        const playBtn = document.getElementById('play-btn');
        playBtn.disabled = false;
        playBtn.onclick = playAudio;
        drawWaveform(buffer);
    }).catch(err => {
        console.error('오디오 디코딩 실패:', err);
        // 폴백: WAV 데이터에서 직접 파형 그리기
        drawWaveformFromBytes(bytes);
        const playBtn = document.getElementById('play-btn');
        playBtn.disabled = false;
        playBtn.onclick = () => {
            const blob = new Blob([bytes], { type: 'audio/wav' });
            const url = URL.createObjectURL(blob);
            const audio = new Audio(url);
            audio.play();
        };
    });
}

function playAudio() {
    if (!currentAudioBuffer || !audioContext) return;

    if (currentSource) {
        try { currentSource.stop(); } catch(e) {}
    }

    currentSource = audioContext.createBufferSource();
    currentSource.buffer = currentAudioBuffer;
    currentSource.connect(audioContext.destination);
    currentSource.start(0);

    const playBtn = document.getElementById('play-btn');
    playBtn.textContent = '재생 중...';
    currentSource.onended = () => {
        playBtn.textContent = '재생';
    };
}

function drawWaveform(audioBuffer) {
    const canvas = document.getElementById('waveform-canvas');
    const dpr = window.devicePixelRatio || 1;
    const w = canvas.clientWidth;
    const h = canvas.clientHeight;
    canvas.width = w * dpr;
    canvas.height = h * dpr;
    const ctx = canvas.getContext('2d');
    ctx.scale(dpr, dpr);

    const data = audioBuffer.getChannelData(0);
    drawWaveformData(ctx, data, w, h);
}

function drawWaveformFromBytes(wavBytes) {
    const canvas = document.getElementById('waveform-canvas');
    const dpr = window.devicePixelRatio || 1;
    const w = canvas.clientWidth;
    const h = canvas.clientHeight;
    canvas.width = w * dpr;
    canvas.height = h * dpr;
    const ctx = canvas.getContext('2d');
    ctx.scale(dpr, dpr);

    // WAV 헤더 건너뛰기 (44바이트), 16-bit PCM 추출
    const view = new DataView(wavBytes.buffer);
    const numSamples = (wavBytes.length - 44) / 2;
    const data = new Float32Array(numSamples);
    for (let i = 0; i < numSamples; i++) {
        data[i] = view.getInt16(44 + i * 2, true) / 32768;
    }
    drawWaveformData(ctx, data, w, h);
}

function drawWaveformData(ctx, data, w, h) {
    ctx.fillStyle = '#16213e';
    ctx.fillRect(0, 0, w, h);

    // 중앙선
    ctx.strokeStyle = '#333';
    ctx.lineWidth = 0.5;
    ctx.beginPath();
    ctx.moveTo(0, h / 2);
    ctx.lineTo(w, h / 2);
    ctx.stroke();

    // 파형
    const step = Math.max(1, Math.floor(data.length / w));
    ctx.strokeStyle = '#4ade80';
    ctx.lineWidth = 1;
    ctx.beginPath();

    for (let x = 0; x < w; x++) {
        const idx = Math.floor(x * data.length / w);
        let min = 1, max = -1;
        for (let j = 0; j < step; j++) {
            const val = data[idx + j] || 0;
            if (val < min) min = val;
            if (val > max) max = val;
        }
        const yMin = (1 - max) * h / 2;
        const yMax = (1 - min) * h / 2;
        ctx.moveTo(x, yMin);
        ctx.lineTo(x, yMax);
    }
    ctx.stroke();

    // 시간 표시
    const duration = data.length / 44100;
    ctx.fillStyle = '#666';
    ctx.font = '10px sans-serif';
    ctx.textAlign = 'left';
    ctx.fillText('0s', 4, h - 4);
    ctx.textAlign = 'right';
    ctx.fillText(`${duration.toFixed(2)}s`, w - 4, h - 4);
    ctx.textAlign = 'center';
    ctx.fillText(`${(duration / 2).toFixed(2)}s`, w / 2, h - 4);
}

document.addEventListener('DOMContentLoaded', loadPipeline);
