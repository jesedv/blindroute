#![forbid(unsafe_code)]

use crate::dft::{fft, ifft};
use crate::params;
use num_complex::Complex64;
use std::f64::consts::PI;

pub fn encode_real(message: &[f64], scale: f64) -> Vec<u64> {
    let n = params::N;
    let n2 = n / 2;
    assert_eq!(message.len(), n2, "message must have N/2 = {n2} real entries");

    let mut v = vec![Complex64::new(0.0, 0.0); n];
    for j in 0..n2 {
        v[j] = Complex64::new(message[j], 0.0);
    }
    for j in 0..n2 {
        v[n - 1 - j] = v[j].conj();
    }

    let mut twisted = v;
    ifft(&mut twisted);

    let zeta_inv = Complex64::new((PI / n as f64).cos(), -(PI / n as f64).sin());
    let mut poly = vec![0u64; n];
    let mut power = Complex64::new(1.0, 0.0);
    for j in 0..n {
        let coeff = twisted[j] * power;
        let val = (coeff.re * scale).round() as i64;
        if val >= 0 {
            poly[j] = val as u64 % params::Q;
        } else {
            poly[j] = params::Q - (((-val) as u64) % params::Q);
        }
        power *= zeta_inv;
    }
    poly
}

pub fn decode_real(poly: &[u64], scale: f64) -> Vec<f64> {
    let n = params::N;
    let n2 = n / 2;
    assert_eq!(poly.len(), n);

    let icoeffs: Vec<i64> = poly
        .iter()
        .map(|&x| {
            let half = params::Q / 2;
            if x <= half {
                x as i64
            } else {
                -((params::Q - x) as i64)
            }
        })
        .collect();

    let zeta = Complex64::new((PI / n as f64).cos(), (PI / n as f64).sin());
    let mut twisted = vec![Complex64::new(0.0, 0.0); n];
    let mut power = Complex64::new(1.0, 0.0);
    for j in 0..n {
        twisted[j] = Complex64::new(icoeffs[j] as f64, 0.0) * power;
        power *= zeta;
    }

    fft(&mut twisted);

    let mut result = vec![0.0f64; n2];
    for j in 0..n2 {
        result[j] = twisted[j].re / scale;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_roundtrip() {
        let n2 = params::N / 2;
        let scale = params::DELTA as f64;
        let msg: Vec<f64> = (0..n2).map(|i| (i as f64 / n2 as f64 - 0.5) * 10.0).collect();

        let poly = encode_real(&msg, scale);
        let decoded = decode_real(&poly, scale);

        let mut max_err = 0.0f64;
        for (_, (&a, &b)) in msg.iter().zip(decoded.iter()).enumerate() {
            let err = (a - b).abs();
            max_err = max_err.max(err);
        }
        assert!(
            max_err < 1e-6,
            "max error {max_err} too large (scale={scale})"
        );
    }

    #[test]
    fn two_way_identity_small() {
        let n2 = params::N / 2;
        let scale = params::DELTA as f64;
        let msg: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let padded: Vec<f64> = msg.iter().copied().chain(std::iter::repeat(0.0)).take(n2).collect();
        let poly = encode_real(&padded, scale);
        let decoded = decode_real(&poly, scale);
        for i in 0..5 {
            let err = (msg[i] - decoded[i]).abs();
            assert!(err < 2e-7, "at {}: {} vs {}", i, msg[i], decoded[i]);
        }
    }

    #[test]
    fn encoding_preserves_realness() {
        let n = params::N;
        let n2 = n / 2;
        let scale = params::DELTA as f64;
        let msg: Vec<f64> = (0..n2).map(|i| (i as f64).sin()).collect();

        let poly = encode_real(&msg, scale);
        let icoeffs: Vec<i64> = poly.iter().map(|&x| {
            let half = params::Q / 2;
            if x <= half { x as i64 } else { -((params::Q - x) as i64) }
        }).collect();

        let zeta = Complex64::new((PI / n as f64).cos(), (PI / n as f64).sin());
        let mut power = Complex64::new(1.0, 0.0);
        let mut max_imag = 0.0f64;
        for j in 0..n {
            let twisted = Complex64::new(icoeffs[j] as f64, 0.0) * power;
            let orig = twisted * power.inv();
            max_imag = max_imag.max(orig.im.abs());
            power *= zeta;
        }
        assert!(max_imag < 1e-6, "polynomial should be real, max imag part {max_imag}");
    }
}
