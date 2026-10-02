//! PROFILING ONLY (#623). Wall-clock counters, printed when MONTAGENT_PROFILE is set.
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
use std::time::Instant;

pub static FRAME_AT_CALLS: AtomicU64 = AtomicU64::new(0);
pub static FRAME_AT_NS: AtomicU64 = AtomicU64::new(0);
pub static SPAWNS: AtomicU64 = AtomicU64::new(0);
pub static SPAWN_NS: AtomicU64 = AtomicU64::new(0);
pub static FALLBACKS: AtomicU64 = AtomicU64::new(0);
pub static FRAMES_PIPED: AtomicU64 = AtomicU64::new(0);
pub static PAINT_NS: AtomicU64 = AtomicU64::new(0);
pub static READBACK_NS: AtomicU64 = AtomicU64::new(0);
pub static PUSH_NS: AtomicU64 = AtomicU64::new(0);
pub static SEAL_NS: AtomicU64 = AtomicU64::new(0);
pub static FRAMES: AtomicU64 = AtomicU64::new(0);

pub fn add(counter: &AtomicU64, since: Instant) {
    counter.fetch_add(since.elapsed().as_nanos() as u64, Relaxed);
}

pub fn dump(total: std::time::Duration) {
    if std::env::var_os("MONTAGENT_PROFILE").is_none() {
        return;
    }
    let ms = |c: &AtomicU64| c.load(Relaxed) as f64 / 1e6;
    let frames = FRAMES.load(Relaxed).max(1) as f64;
    let line = |name: &str, c: &AtomicU64| {
        eprintln!(
            "PROFILE {name:<12} {:>10.1} ms total {:>8.2} ms/frame",
            ms(c),
            ms(c) / frames
        );
    };
    eprintln!("PROFILE frames {} wall {:.1} ms", frames, total.as_secs_f64() * 1e3);
    line("paint", &PAINT_NS);
    line("  frame_at", &FRAME_AT_NS);
    line("  spawns", &SPAWN_NS);
    line("readback", &READBACK_NS);
    line("push", &PUSH_NS);
    line("seal", &SEAL_NS);
    eprintln!(
        "PROFILE frame_at calls {} spawns {} fallbacks {} frames piped back {}",
        FRAME_AT_CALLS.load(Relaxed),
        SPAWNS.load(Relaxed),
        FALLBACKS.load(Relaxed),
        FRAMES_PIPED.load(Relaxed)
    );
}
