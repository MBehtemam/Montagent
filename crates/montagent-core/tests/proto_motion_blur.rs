//! PROTOTYPE #718 — throwaway. The byte-identity sweep for motion blur: the frames at the
//! encoder's input, hashed, at one painter against K = 1..10 painters over several chunk
//! sizes, with the blur and shadow bounds hint (#652) on and off.
//!
//! ```sh
//! MB718_PROJECT=$PWD/prototypes/motion-blur/motion-blur.montagent.json \
//!   cargo test -p montagent-core --test proto_motion_blur -- --ignored --nocapture
//! ```

use std::path::PathBuf;
use std::sync::atomic::Ordering;

use montagent_core::report::ExitCode;
use montagent_core::verbs::render::{Ask, Forced, Progress, force_painting, render, tap_frames};
use montagent_render::canvas::{HINT_FIRES, bound_filter_layers};

fn run(path: &PathBuf, forced: Option<Forced>, hint: bool) -> (Vec<u64>, usize, u128) {
    let path = path.clone();
    std::thread::spawn(move || {
        bound_filter_layers(hint);
        let _forced = forced.map(force_painting);
        let tap = tap_frames();
        let before = HINT_FIRES.load(Ordering::Relaxed);
        let clock = std::time::Instant::now();
        let answer = render(&path, &Ask::default(), &mut |_: Progress| {});
        let wall = clock.elapsed().as_millis();
        assert_eq!(
            answer.report().exit_code(),
            ExitCode::Ok,
            "{}",
            answer.to_json()
        );
        (tap.hashes(), HINT_FIRES.load(Ordering::Relaxed) - before, wall)
    })
    .join()
    .expect("the render thread")
}

#[test]
#[ignore]
fn motion_blur_sweep() {
    let path = PathBuf::from(std::env::var("MB718_PROJECT").expect("MB718_PROJECT"));
    let (baseline, fires, wall) = run(&path, Some(Forced::OnePainter), false);
    println!(
        "{{\"run\":\"baseline\",\"painters\":1,\"hint\":false,\"frames\":{},\"hint_fires\":{fires},\"wall_ms\":{wall}}}",
        baseline.len()
    );
    let settings: [(usize, u64); 10] = [
        (1, 1000),
        (2, 5),
        (3, 2),
        (4, 1),
        (5, 3),
        (6, 2),
        (7, 4),
        (8, 3),
        (9, 1),
        (10, 7),
    ];
    let mut all_equal = true;
    for hint in [false, true] {
        let mut runs: Vec<(String, Option<Forced>)> = vec![("one painter".into(), Some(Forced::OnePainter))];
        for (k, c) in settings {
            runs.push((
                format!("{k}x{c}"),
                Some(Forced::Chunks {
                    painters: k,
                    chunk: c,
                    window_bytes: None,
                }),
            ));
        }
        runs.push(("default".into(), None));
        for (name, forced) in runs {
            if name == "one painter" && !hint {
                continue;
            }
            let (frames, fires, wall) = run(&path, forced, hint);
            let differ: Vec<usize> = (0..baseline.len().max(frames.len()))
                .filter(|&i| baseline.get(i) != frames.get(i))
                .collect();
            all_equal &= differ.is_empty();
            println!(
                "{{\"run\":\"{name}\",\"hint\":{hint},\"frames\":{},\"differing\":{},\"first_differing\":{:?},\"hint_fires\":{fires},\"wall_ms\":{wall}}}",
                frames.len(),
                differ.len(),
                differ.first()
            );
        }
    }
    let digest: u64 = baseline
        .iter()
        .fold(0xcbf29ce484222325, |h, f| (h ^ f).wrapping_mul(0x100000001b3));
    println!("{{\"baseline_digest\":\"{digest:016x}\",\"all_equal\":{all_equal}}}");
    assert!(all_equal, "a sweep run differs from the one-painter baseline");
}

/// The frame hashes of one project at one painter, printed, for comparing two projects.
#[test]
#[ignore]
fn motion_blur_hashes() {
    let path = PathBuf::from(std::env::var("MB718_PROJECT").expect("MB718_PROJECT"));
    let (frames, _, wall) = run(&path, Some(Forced::OnePainter), true);
    let digest: u64 = frames
        .iter()
        .fold(0xcbf29ce484222325, |h, f| (h ^ f).wrapping_mul(0x100000001b3));
    println!(
        "{{\"project\":{:?},\"frames\":{},\"digest\":\"{digest:016x}\",\"wall_ms\":{wall},\"hashes\":{:?}}}",
        path.file_name().unwrap_or_default(),
        frames.len(),
        frames
    );
}
