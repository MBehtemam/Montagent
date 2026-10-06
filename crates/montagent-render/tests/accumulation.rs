//! The motion-blur average (ADR-0155 §4): integer, in fixed order, correctly rounded, so
//! that N identical samples give back exactly the source bytes.

use montagent_render::canvas::Accumulation;

/// Every premultiplied RGBA byte pattern a pixel can hold with `a` as its alpha, plus a few
/// extremes: the source a sample would read back.
fn source() -> Vec<u8> {
    let mut bytes = Vec::new();
    for a in [0u8, 1, 2, 127, 128, 254, 255] {
        for c in [0u8, 1, a / 2, a.saturating_sub(1), a] {
            bytes.extend([c.min(a), (c / 3).min(a), a.saturating_sub(c).min(a), a]);
        }
    }
    bytes
}

#[test]
fn n_identical_samples_average_back_to_exactly_the_source_bytes() {
    let source = source();
    for samples in [2, 3, 4, 7, 8, 16, 31, 32] {
        let mut sum = Accumulation::new(source.len());
        for _ in 0..samples {
            sum.add(&source);
        }
        assert_eq!(sum.mean(), source, "N = {samples}");
    }
}

#[test]
fn a_tie_rounds_half_up() {
    // (0 + 1) / 2 = 0.5 → 1; (1 + 2) / 2 = 1.5 → 2; (0 + 0 + 0 + 2) / 4 = 0.5 → 1;
    // (0 + 0 + 1) / 3 = 0.33 → 0; (0 + 1 + 1) / 3 = 0.67 → 1.
    let mut two = Accumulation::new(4);
    two.add(&[0, 1, 0, 255]);
    two.add(&[1, 2, 0, 255]);
    assert_eq!(two.mean(), vec![1, 2, 0, 255]);

    let mut four = Accumulation::new(1);
    for byte in [0, 0, 0, 2] {
        four.add(&[byte]);
    }
    assert_eq!(four.mean(), vec![1]);

    let mut three = Accumulation::new(2);
    three.add(&[0, 0]);
    three.add(&[0, 1]);
    three.add(&[1, 1]);
    assert_eq!(three.mean(), vec![0, 1]);
}

#[test]
fn the_mean_of_premultiplied_samples_is_premultiplied() {
    // A colour byte never exceeds its alpha in any sample, so it never does in the mean.
    let mut sum = Accumulation::new(8);
    sum.add(&[255, 0, 0, 255, 0, 0, 0, 0]);
    sum.add(&[0, 0, 0, 0, 3, 3, 3, 3]);
    sum.add(&[0, 0, 0, 0, 0, 0, 0, 0]);
    let mean = sum.mean();
    assert_eq!(mean, vec![85, 0, 0, 85, 1, 1, 1, 1]);
    for pixel in mean.chunks_exact(4) {
        assert!(pixel[..3].iter().all(|&c| c <= pixel[3]), "{pixel:?}");
    }
}
