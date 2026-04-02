"""
GPU 학습 스크립트 (RTX 8000 / CUDA 최적화)

사용법:
  source .venv/bin/activate
  python train_gpu.py --dataset_dir training_data --batch_size 48 --epochs 1000

RTX 8000 (48GB VRAM) 권장 설정:
  --batch_size 48   (VRAM 여유 있으면 64까지)
  --epochs 1000     (2000샘플 기준 8~15시간)
  --fp16            (FP16 혼합 정밀도 — 2배 빠름)
"""
import os
import sys
import argparse
import time

def main():
    parser = argparse.ArgumentParser(description="GPU 학습 (RTX 8000 / CUDA)")
    parser.add_argument("--dataset_dir", default="training_data")
    parser.add_argument("--output_dir", default="trained_models")
    parser.add_argument("--epochs", type=int, default=1000)
    parser.add_argument("--batch_size", type=int, default=48)
    parser.add_argument("--fp16", action="store_true", default=True, help="FP16 혼합 정밀도 (기본 활성)")
    parser.add_argument("--no-fp16", dest="fp16", action="store_false")
    parser.add_argument("--resume", type=str, default=None, help="체크포인트에서 재개")
    parser.add_argument("--lr", type=float, default=0.0002, help="학습률")
    args = parser.parse_args()

    # GPU 확인
    import torch
    if not torch.cuda.is_available():
        print("에러: CUDA GPU를 찾을 수 없습니다.")
        print(f"  PyTorch: {torch.__version__}")
        print(f"  CUDA built: {torch.version.cuda}")
        sys.exit(1)

    gpu_name = torch.cuda.get_device_name(0)
    vram_gb = torch.cuda.get_device_properties(0).total_memory / 1e9
    print(f"GPU: {gpu_name} ({vram_gb:.0f}GB VRAM)")
    print(f"설정: batch_size={args.batch_size}, epochs={args.epochs}, fp16={args.fp16}")

    # VRAM에 따라 batch_size 자동 조정 경고
    if vram_gb < 16 and args.batch_size > 16:
        print(f"경고: VRAM {vram_gb:.0f}GB에서 batch_size {args.batch_size}는 OOM 위험. 16 이하 권장.")
    elif vram_gb < 24 and args.batch_size > 32:
        print(f"경고: VRAM {vram_gb:.0f}GB에서 batch_size {args.batch_size}는 OOM 위험. 32 이하 권장.")

    # 데이터 확인
    metadata = os.path.join(args.dataset_dir, "metadata.csv")
    if not os.path.exists(metadata):
        print(f"\n에러: {metadata}가 없습니다.")
        print(f"학습 데이터 준비:")
        print(f"  {args.dataset_dir}/wavs/  — WAV (22050Hz, mono, 16-bit, 2~10초)")
        print(f"  {args.dataset_dir}/metadata.csv — 파일명|텍스트|텍스트")
        sys.exit(1)

    with open(metadata) as f:
        lines = [l.strip() for l in f if l.strip()]

    if len(lines) < 10:
        print(f"\n에러: 학습 데이터가 {len(lines)}개뿐입니다. 최소 50개 필요.")
        sys.exit(1)

    print(f"학습 데이터: {len(lines)}개 샘플")

    # 예상 시간
    samples_per_epoch = len(lines)
    batches_per_epoch = samples_per_epoch / args.batch_size
    # RTX 8000 기준: batch당 약 0.3초 (fp16), 0.5초 (fp32)
    sec_per_batch = 0.3 if args.fp16 else 0.5
    total_hours = (batches_per_epoch * args.epochs * sec_per_batch) / 3600
    print(f"예상 학습 시간: {total_hours:.1f}시간 ({gpu_name} 기준)")

    # Coqui TTS import
    from trainer import Trainer, TrainerArgs
    from TTS.tts.configs.shared_configs import BaseDatasetConfig
    from TTS.tts.configs.vits_config import VitsConfig
    from TTS.tts.datasets import load_tts_samples
    from TTS.tts.models.vits import Vits, VitsAudioConfig
    from TTS.tts.utils.text.tokenizer import TTSTokenizer
    from TTS.utils.audio import AudioProcessor

    os.makedirs(args.output_dir, exist_ok=True)

    dataset_config = BaseDatasetConfig(
        formatter="ljspeech",
        meta_file_train="metadata.csv",
        path=os.path.abspath(args.dataset_dir),
    )

    audio_config = VitsAudioConfig(
        sample_rate=22050,
        win_length=1024,
        hop_length=256,
        num_mels=80,
        mel_fmin=0,
        mel_fmax=None,
    )

    config = VitsConfig(
        audio=audio_config,
        run_name="custom_korean_vits",
        batch_size=args.batch_size,
        eval_batch_size=min(args.batch_size // 2, 16),
        batch_group_size=5,
        num_loader_workers=8,        # GPU 학습: 워커 늘림
        num_eval_loader_workers=4,
        run_eval=True,
        test_delay_epochs=-1,
        epochs=args.epochs,
        text_cleaner="basic_cleaners",
        use_phonemes=True,
        phoneme_language="ko",
        phonemizer="espeak",
        phoneme_cache_path=os.path.join(args.output_dir, "phoneme_cache"),
        compute_input_seq_cache=True,
        print_step=25,
        print_eval=True,
        mixed_precision=args.fp16,   # FP16 혼합 정밀도
        output_path=args.output_dir,
        datasets=[dataset_config],
        cudnn_benchmark=True,        # GPU: cuDNN 자동 최적화
        lr=args.lr,
    )

    ap = AudioProcessor.init_from_config(config)
    tokenizer, config = TTSTokenizer.init_from_config(config)

    train_samples, eval_samples = load_tts_samples(
        dataset_config,
        eval_split=True,
        eval_split_max_size=config.eval_split_max_size,
        eval_split_size=config.eval_split_size,
    )

    model = Vits(config, ap, tokenizer, speaker_manager=None)

    trainer_args = TrainerArgs()
    if args.resume:
        trainer_args.continue_path = args.resume
        print(f"체크포인트에서 재개: {args.resume}")

    trainer = Trainer(
        trainer_args,
        config,
        args.output_dir,
        model=model,
        train_samples=train_samples,
        eval_samples=eval_samples,
    )

    print(f"\n{'='*50}")
    print(f"  학습 시작")
    print(f"  GPU: {gpu_name}")
    print(f"  샘플: {len(lines)}개, {args.epochs} epochs")
    print(f"  batch: {args.batch_size}, FP16: {args.fp16}")
    print(f"  출력: {args.output_dir}")
    print(f"{'='*50}\n")

    start = time.time()
    trainer.fit()
    elapsed = time.time() - start

    print(f"\n학습 완료: {elapsed/3600:.1f}시간")
    print(f"모델: {args.output_dir}")
    print(f"\n합성 테스트:")
    print(f"  tts --text '안녕하세요' --model_path {args.output_dir}/best_model.pth --config_path {args.output_dir}/config.json --out_path test.wav")

if __name__ == "__main__":
    main()
