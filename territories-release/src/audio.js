import { useCallback, useEffect, useRef } from 'react';

export function playTone(audioCtxRef, kind) {
  try {
    const AudioCtx = window.AudioContext || window.webkitAudioContext;
    if (!AudioCtx) return;
    if (!audioCtxRef.current) {
      audioCtxRef.current = new AudioCtx();
    }
    const ctx = audioCtxRef.current;
    if (ctx.state === 'suspended') {
      ctx.resume().catch(() => {});
    }
    const now = ctx.currentTime;

    const emit = (freqStart, freqEnd, dur, gainPeak, type = 'triangle', offset = 0) => {
      const osc = ctx.createOscillator();
      const gain = ctx.createGain();
      osc.type = type;
      osc.frequency.setValueAtTime(freqStart, now + offset);
      osc.frequency.exponentialRampToValueAtTime(Math.max(40, freqEnd), now + offset + dur);
      gain.gain.setValueAtTime(0.0001, now + offset);
      gain.gain.linearRampToValueAtTime(gainPeak, now + offset + 0.006);
      gain.gain.exponentialRampToValueAtTime(0.0001, now + offset + dur);
      osc.connect(gain);
      gain.connect(ctx.destination);
      osc.start(now + offset);
      osc.stop(now + offset + dur + 0.01);
    };

    if (kind === 'place') {
      emit(240, 125, 0.055, 0.055, 'triangle');
    } else if (kind === 'capture') {
      emit(440, 440, 0.11, 0.045, 'sine', 0);
      emit(660, 660, 0.14, 0.04, 'sine', 0.045);
    } else if (kind === 'illegal') {
      emit(120, 82, 0.05, 0.04, 'sine');
    } else if (kind === 'gameover') {
      emit(392, 392, 0.14, 0.045, 'sine', 0);
      emit(523.25, 523.25, 0.18, 0.045, 'sine', 0.08);
    }
  } catch {

  }
}

export function useSoundEffects(soundEnabled) {
  const audioCtxRef = useRef(null);
  const soundEnabledRef = useRef(soundEnabled);

  useEffect(() => {
    soundEnabledRef.current = soundEnabled;
  }, [soundEnabled]);

  return useCallback((kind) => {
    if (!soundEnabledRef.current) return;
    playTone(audioCtxRef, kind);
  }, []);
}
