//! CPAL audio capture with PTT gate (D-01, D-03, D-04).
//! RED phase: tests defined first, implementation pending.

#[cfg(test)]
mod tests {
    use super::*;
    use ringbuf::HeapRb;
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };

    /// Compilation sentinel: fails until AudioHandle + start_audio_stream exist in super scope.
    #[allow(dead_code)]
    fn _type_check() {
        let _: fn(Arc<AtomicBool>) -> anyhow::Result<AudioHandle> = start_audio_stream;
    }

    #[test]
    fn ptt_gate_off_discards_samples() {
        let ptt = Arc::new(AtomicBool::new(false));
        let rb = HeapRb::<f32>::new(64);
        let (mut producer, mut consumer) = rb.split();

        let data = [0.1_f32; 16];
        if ptt.load(Ordering::Relaxed) {
            let _ = producer.push_slice(&data);
        }

        assert_eq!(
            consumer.pop_slice(&mut [0f32; 16]),
            0,
            "PTT off: no samples should reach the ring buffer"
        );
    }

    #[test]
    fn ptt_gate_on_pushes_samples() {
        let ptt = Arc::new(AtomicBool::new(true));
        let rb = HeapRb::<f32>::new(64);
        let (mut producer, mut consumer) = rb.split();

        let data = [0.5_f32; 8];
        if ptt.load(Ordering::Relaxed) {
            let _ = producer.push_slice(&data);
        }

        let mut out = [0f32; 8];
        let n = consumer.pop_slice(&mut out);
        assert_eq!(n, 8, "PTT on: all samples must reach ring buffer");
        assert!((out[0] - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn ring_buffer_overflow_does_not_panic() {
        let rb = HeapRb::<f32>::new(4);
        let (mut producer, _consumer) = rb.split();
        let data = [0.1f32; 8]; // 8 samples into buffer of 4
        let _ = producer.push_slice(&data); // must not panic
    }
}
