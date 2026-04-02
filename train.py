"""
커스텀 한국어 VITS 학습 스크립트
사용법: source .venv/bin/activate && python train.py --dataset_dir training_data
"""
import os
import argparse

def main():
    parser = argparse.ArgumentParser(description="커스텀 한국어 VITS 학습")
    parser.add_argument("--dataset_dir", default="training_data", help="학습 데이터 디렉토리")
    parser.add_argument("--epochs", type=int, default=1000)
    parser.add_argument("--batch_size", type=int, default=8)
    parser.add_argument("--output_dir", default="trained_models")
    args = parser.parse_args()

    # 데이터 확인
    metadata = os.path.join(args.dataset_dir, "metadata.csv")
    if not os.path.exists(metadata):
        print(f"에러: {metadata}가 없습니다.")
        print(f"먼저 학습 데이터를 준비하세요:")
        print(f"  {args.dataset_dir}/wavs/ — WAV 파일 (22050Hz, mono, 16-bit)")
        print(f"  {args.dataset_dir}/metadata.csv — 파일명|텍스트|텍스트")
        return

    with open(metadata) as f:
        lines = [l.strip() for l in f if l.strip()]
    print(f"학습 데이터: {len(lines)}개 샘플")

    if len(lines) < 50:
        print(f"경고: 최소 50개 이상의 샘플이 필요합니다 (현재 {len(lines)}개)")
        print(f"권장: 500개 이상 (약 30분 이상의 녹음)")
        return

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
        eval_batch_size=4,
        batch_group_size=5,
        num_loader_workers=2,
        num_eval_loader_workers=1,
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
        mixed_precision=False,
        output_path=args.output_dir,
        datasets=[dataset_config],
        cudnn_benchmark=False,
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

    trainer = Trainer(
        TrainerArgs(),
        config,
        args.output_dir,
        model=model,
        train_samples=train_samples,
        eval_samples=eval_samples,
    )

    print(f"\n학습 시작: {len(lines)}개 샘플, {args.epochs} epochs, batch_size={args.batch_size}")
    print(f"출력: {args.output_dir}")
    print(f"경고: CPU 학습은 매우 느립니다. GPU 사용을 권장합니다.\n")

    trainer.fit()

if __name__ == "__main__":
    main()
