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
    btn.textContent = '생성 중...';

    try {
        const res = await fetch(`${API_BASE}/api/synthesize`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ text, language: lang, params: getTuningParams() }),
        });

        const data = await res.json();
        renderResults(data);

        // Step 2, 3 표시
        document.getElementById('listen-section').style.display = 'block';
        document.getElementById('tuning-section').style.display = 'block';
    } catch (e) {
        console.error('합성 실패:', e);
        alert('합성 요청에 실패했습니다.');
    } finally {
        btn.disabled = false;
        btn.textContent = '합성';
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
    // 파이프라인 결과 (상세 분석 패널)
    const container = document.getElementById('pipeline-results');
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

    if (data.audio_base64) {
        lastWavBase64 = data.audio_base64;
        document.getElementById('download-btn').disabled = false;
        // A/B 비교를 위해 이전 오디오 저장
        if (currentAudioBase64) {
            previousAudioBase64 = currentAudioBase64;
            const abBtn = document.getElementById('ab-btn');
            abBtn.style.display = 'inline-block';
            abBtn.disabled = false;
            abBtn.textContent = 'A/B: 현재 (A)';
            isPlayingA = true;
        }
        currentAudioBase64 = data.audio_base64;
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
        lastAudioData = buffer.getChannelData(0);
        const playBtn = document.getElementById('play-btn');
        playBtn.disabled = false;
        playBtn.onclick = playAudio;
        drawWaveform(buffer);
        showQualitySummary(lastAudioData);
    }).catch(err => {
        console.error('오디오 디코딩 실패:', err);
        // 폴백: WAV 데이터에서 직접 파형 그리기
        const view = new DataView(bytes.buffer);
        const numSamples = (bytes.length - 44) / 2;
        lastAudioData = new Float32Array(numSamples);
        for (let i = 0; i < numSamples; i++) {
            lastAudioData[i] = view.getInt16(44 + i * 2, true) / 32768;
        }
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

// === 튜닝 기능 ===

function getTuningParams() {
    return {
        base_pitch_hz: parseFloat(document.getElementById('pitch-slider').value),
        speed_factor: parseFloat(document.getElementById('speed-slider').value),
        volume: parseFloat(document.getElementById('volume-slider').value),
    };
}

function updateTuningValue(param) {
    const slider = document.getElementById(`${param}-slider`);
    const display = document.getElementById(`${param}-value`);
    display.textContent = slider.value;
    onSliderChange();
}

const PRESETS = {
    default: { base_pitch_hz: 150, speed_factor: 1.0, volume: 1.0 },
    high:    { base_pitch_hz: 220, speed_factor: 1.0, volume: 1.0 },
    low:     { base_pitch_hz: 100, speed_factor: 1.0, volume: 1.0 },
    fast:    { base_pitch_hz: 150, speed_factor: 1.5, volume: 1.0 },
    slow:    { base_pitch_hz: 150, speed_factor: 0.7, volume: 1.0 },
};

function applyPreset(name) {
    const preset = PRESETS[name];
    if (!preset) return;
    document.getElementById('pitch-slider').value = preset.base_pitch_hz;
    document.getElementById('speed-slider').value = preset.speed_factor;
    document.getElementById('volume-slider').value = preset.volume;
    updateTuningValue('pitch');
    updateTuningValue('speed');
    updateTuningValue('volume');
}

function saveCustomPreset() {
    const params = getTuningParams();
    localStorage.setItem('tts-custom-preset', JSON.stringify(params));
    document.getElementById('load-preset-btn').disabled = false;
}

function loadCustomPreset() {
    const saved = localStorage.getItem('tts-custom-preset');
    if (!saved) return;
    const params = JSON.parse(saved);
    document.getElementById('pitch-slider').value = params.base_pitch_hz;
    document.getElementById('speed-slider').value = params.speed_factor;
    document.getElementById('volume-slider').value = params.volume;
    updateTuningValue('pitch');
    updateTuningValue('speed');
    updateTuningValue('volume');
}

// A/B 비교용 이전 오디오 저장
let previousAudioBase64 = null;
let currentAudioBase64 = null;
let isPlayingA = true;

function toggleABCompare() {
    if (!previousAudioBase64 || !currentAudioBase64) return;
    isPlayingA = !isPlayingA;
    const btn = document.getElementById('ab-btn');
    if (isPlayingA) {
        btn.textContent = 'A/B: 현재 (A)';
        setupAudioPlayback(currentAudioBase64);
    } else {
        btn.textContent = 'A/B: 이전 (B)';
        setupAudioPlayback(previousAudioBase64);
    }
}

// === 스펙트로그램 + 탭 전환 ===

let lastAudioData = null; // Float32Array of decoded audio

function switchViz(mode) {
    const wfCanvas = document.getElementById('waveform-canvas');
    const sgCanvas = document.getElementById('spectrogram-canvas');
    const qPanel = document.getElementById('quality-panel');
    document.querySelectorAll('.viz-tab').forEach(t => t.classList.remove('active'));

    wfCanvas.style.display = 'none';
    sgCanvas.style.display = 'none';
    qPanel.style.display = 'none';

    if (mode === 'spectrogram') {
        sgCanvas.style.display = 'block';
        document.querySelectorAll('.viz-tab')[1].classList.add('active');
        if (lastAudioData) drawSpectrogram(lastAudioData);
    } else if (mode === 'quality') {
        qPanel.style.display = 'block';
        document.querySelectorAll('.viz-tab')[2].classList.add('active');
        if (lastAudioData) runQualityCheck(lastAudioData);
    } else {
        wfCanvas.style.display = 'block';
        document.querySelectorAll('.viz-tab')[0].classList.add('active');
    }
}

// === 품질 검증 ===
function runQualityCheck(audioData) {
    const n = audioData.length;
    const sr = 44100;
    const container = document.getElementById('quality-results');

    // 1. 클릭 감지
    let clicks = 0;
    for (let i = 1; i < n; i++) {
        if (Math.abs(audioData[i] - audioData[i-1]) > 0.3) clicks++;
    }

    // 2. 무음 비율
    let silence = 0;
    for (let i = 0; i < n; i++) {
        if (Math.abs(audioData[i]) < 0.003) silence++;
    }
    const silencePct = (silence / n * 100).toFixed(1);

    // 3. 고주파 잡음 비율 (50ms 윈도우)
    const winSize = Math.floor(sr * 0.05);
    let noisyWindows = 0, totalWindows = 0;
    for (let w = 0; w < n - winSize; w += winSize) {
        let rms = 0, hf = 0;
        for (let i = w; i < w + winSize; i++) {
            rms += audioData[i] * audioData[i];
            if (i > w) hf += (audioData[i] - audioData[i-1]) ** 2;
        }
        rms = Math.sqrt(rms / winSize);
        hf = Math.sqrt(hf / (winSize - 1));
        totalWindows++;
        if (rms > 0.005 && hf / (rms + 0.001) > 0.4) noisyWindows++;
    }

    // 4. RMS
    let totalRms = 0;
    for (let i = 0; i < n; i++) totalRms += audioData[i] ** 2;
    totalRms = Math.sqrt(totalRms / n);

    // 5. 동적 범위
    let maxVal = 0;
    for (let i = 0; i < n; i++) maxVal = Math.max(maxVal, Math.abs(audioData[i]));
    const crestFactor = maxVal / (totalRms + 0.001);

    // 판정
    const clickGrade = clicks === 0 ? '✅ 없음' : clicks < 5 ? '⚠️ ' + clicks + '개' : '❌ ' + clicks + '개';
    const silenceGrade = silencePct < 15 ? '✅' : silencePct < 30 ? '⚠️' : '❌';
    const noiseGrade = noisyWindows === 0 ? '✅ 없음' : noisyWindows < 3 ? '⚠️ ' + noisyWindows + '구간' : '❌ ' + noisyWindows + '구간';
    const crestGrade = crestFactor < 6 ? '✅' : crestFactor < 10 ? '⚠️' : '❌';

    container.innerHTML = `
        <div class="quality-item">
            <div class="q-label">클릭/크래클링</div>
            <div class="q-value">${clickGrade}</div>
            <div class="q-detail">인접 샘플 급변 (>0.3) 횟수</div>
        </div>
        <div class="quality-item">
            <div class="q-label">무음 비율</div>
            <div class="q-value">${silenceGrade} ${silencePct}%</div>
            <div class="q-detail">목표: 15% 이하 (pause 포함)</div>
        </div>
        <div class="quality-item">
            <div class="q-label">잡음 구간</div>
            <div class="q-value">${noiseGrade}</div>
            <div class="q-detail">고주파 에너지 >40% 구간 수 (총 ${totalWindows}개 중)</div>
        </div>
        <div class="quality-item">
            <div class="q-label">동적 범위</div>
            <div class="q-value">${crestGrade} ${crestFactor.toFixed(1)}x</div>
            <div class="q-detail">max/rms 비율 (목표: 6x 이하)</div>
        </div>
        <div class="quality-item">
            <div class="q-label">RMS 레벨</div>
            <div class="q-value">${(totalRms * 100).toFixed(1)}%</div>
            <div class="q-detail">전체 평균 에너지</div>
        </div>
        <div class="quality-item">
            <div class="q-label">길이</div>
            <div class="q-value">${(n / sr).toFixed(2)}초</div>
            <div class="q-detail">${n.toLocaleString()} 샘플</div>
        </div>
    `;
}

// === WAV 다운로드 ===
let lastWavBase64 = null;

function downloadWav() {
    if (!lastWavBase64) return;
    const binary = atob(lastWavBase64);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
    const blob = new Blob([bytes], { type: 'audio/wav' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'tts_output.wav';
    a.click();
    URL.revokeObjectURL(url);
}

// === 자동 재합성 ===
let autoResynthEnabled = false;
let autoResynthTimer = null;

function toggleAutoResynth() {
    autoResynthEnabled = document.getElementById('auto-resynth').checked;
}

function onSliderChange() {
    if (!autoResynthEnabled) return;
    if (autoResynthTimer) clearTimeout(autoResynthTimer);
    autoResynthTimer = setTimeout(() => {
        const text = document.getElementById('text-input').value.trim();
        if (text) synthesize();
    }, 300); // 300ms 디바운스
}

function drawSpectrogram(audioData) {
    const canvas = document.getElementById('spectrogram-canvas');
    const dpr = window.devicePixelRatio || 1;
    const w = canvas.clientWidth;
    const h = canvas.clientHeight;
    canvas.width = w * dpr;
    canvas.height = h * dpr;
    const ctx = canvas.getContext('2d');
    ctx.scale(dpr, dpr);

    ctx.fillStyle = '#0a0a1a';
    ctx.fillRect(0, 0, w, h);

    const fftSize = 512;
    const halfFFT = fftSize / 2;
    const hopSize = Math.max(1, Math.floor(audioData.length / w));
    const numFrames = Math.min(w, Math.floor(audioData.length / hopSize));

    // Hann 윈도우
    const window = new Float32Array(fftSize);
    for (let i = 0; i < fftSize; i++) {
        window[i] = 0.5 * (1 - Math.cos(2 * Math.PI * i / (fftSize - 1)));
    }

    // 각 프레임의 스펙트럼 계산 (간이 DFT — 실수부만)
    const magnitudes = [];
    let maxMag = 0;

    for (let frame = 0; frame < numFrames; frame++) {
        const offset = frame * hopSize;
        const spectrum = new Float32Array(halfFFT);

        for (let k = 0; k < halfFFT; k++) {
            let re = 0, im = 0;
            for (let n = 0; n < fftSize; n++) {
                const sample = (offset + n < audioData.length) ? audioData[offset + n] * window[n] : 0;
                const angle = -2 * Math.PI * k * n / fftSize;
                re += sample * Math.cos(angle);
                im += sample * Math.sin(angle);
            }
            const mag = Math.sqrt(re * re + im * im);
            spectrum[k] = mag;
            if (mag > maxMag) maxMag = mag;
        }
        magnitudes.push(spectrum);
    }

    // 렌더링
    if (maxMag === 0) maxMag = 1;
    const imgData = ctx.createImageData(numFrames, h);

    for (let x = 0; x < numFrames; x++) {
        const spectrum = magnitudes[x];
        for (let y = 0; y < h; y++) {
            const freqBin = Math.floor((1 - y / h) * halfFFT);
            const mag = spectrum[Math.min(freqBin, halfFFT - 1)] / maxMag;
            // dB 스케일 + 컬러맵 (어두운 파랑 → 노랑 → 빨강)
            const db = Math.max(0, 1 + Math.log10(mag + 0.001) / 3);
            const idx = (y * numFrames + x) * 4;
            if (db < 0.33) {
                imgData.data[idx] = 0;
                imgData.data[idx + 1] = Math.floor(db * 3 * 100);
                imgData.data[idx + 2] = Math.floor(db * 3 * 180);
            } else if (db < 0.66) {
                const t = (db - 0.33) * 3;
                imgData.data[idx] = Math.floor(t * 230);
                imgData.data[idx + 1] = Math.floor(100 + t * 155);
                imgData.data[idx + 2] = Math.floor(180 - t * 140);
            } else {
                const t = (db - 0.66) * 3;
                imgData.data[idx] = Math.floor(230 + t * 25);
                imgData.data[idx + 1] = Math.floor(255 - t * 100);
                imgData.data[idx + 2] = Math.floor(40 - t * 40);
            }
            imgData.data[idx + 3] = 255;
        }
    }

    // 스펙트로그램을 캔버스에 그리기 (스케일링)
    const tmpCanvas = document.createElement('canvas');
    tmpCanvas.width = numFrames;
    tmpCanvas.height = h;
    tmpCanvas.getContext('2d').putImageData(imgData, 0, 0);
    ctx.drawImage(tmpCanvas, 0, 0, w, h);

    // 주파수 축 라벨
    ctx.fillStyle = '#666';
    ctx.font = '10px sans-serif';
    ctx.textAlign = 'left';
    const maxFreq = 44100 / 2;
    for (let f = 0; f <= maxFreq; f += 2000) {
        const y = h * (1 - f / maxFreq);
        ctx.fillText(`${f / 1000}k`, 4, y + 4);
    }

    // 시간 축
    const duration = audioData.length / 44100;
    ctx.textAlign = 'center';
    ctx.fillText(`${duration.toFixed(2)}s`, w / 2, h - 4);
}

// === 추가 함수들 ===

function stopAudio() {
    if (currentSource) {
        try { currentSource.stop(); } catch(e) {}
        currentSource = null;
    }
    document.getElementById('play-btn').textContent = '재생';
}

function showQualitySummary(audioData) {
    const n = audioData.length;
    let silence = 0;
    for (let i = 0; i < n; i++) if (Math.abs(audioData[i]) < 0.003) silence++;
    const silPct = (silence / n * 100).toFixed(0);

    let clicks = 0;
    for (let i = 1; i < n; i++) if (Math.abs(audioData[i] - audioData[i-1]) > 0.3) clicks++;

    const dur = (n / 44100).toFixed(2);
    const el = document.getElementById('quality-summary');
    const silIcon = silPct < 20 ? '✅' : silPct < 35 ? '⚠️' : '❌';
    const clickIcon = clicks === 0 ? '✅' : clicks < 5 ? '⚠️' : '❌';
    el.innerHTML = `${dur}초 | 무음 ${silIcon} ${silPct}% | 클릭 ${clickIcon} ${clicks}개 | <em>마음에 안 들면 아래 슬라이더를 조절 후 다시 합성하세요</em>`;
}

function switchDetail(mode) {
    document.querySelectorAll('.detail-tab').forEach(t => t.classList.remove('active'));
    document.querySelectorAll('.detail-content').forEach(c => c.style.display = 'none');

    if (mode === 'spectrogram') {
        document.querySelectorAll('.detail-tab')[1].classList.add('active');
        document.getElementById('detail-spectrogram').style.display = 'block';
        if (lastAudioData) drawSpectrogram(lastAudioData);
    } else if (mode === 'quality') {
        document.querySelectorAll('.detail-tab')[2].classList.add('active');
        document.getElementById('detail-quality').style.display = 'block';
        if (lastAudioData) runQualityCheck(lastAudioData);
    } else {
        document.querySelectorAll('.detail-tab')[0].classList.add('active');
        document.getElementById('detail-pipeline').style.display = 'block';
    }
}

document.addEventListener('DOMContentLoaded', () => {
    loadPipeline();
});
