use num_complex::Complex64;
use std::f64::consts::PI;

pub fn fft(a: &mut [Complex64]) {
    let n = a.len();
    debug_assert!(n.is_power_of_two());
    bit_reverse(a);
    let mut len = 1;
    while len < n {
        let wlen = Complex64::new(0.0, 2.0 * PI / (2 * len) as f64).exp();
        for start in (0..n).step_by(2 * len) {
            let mut w = Complex64::new(1.0, 0.0);
            for j in 0..len {
                let u = a[start + j];
                let v = a[start + j + len] * w;
                a[start + j] = u + v;
                a[start + j + len] = u - v;
                w *= wlen;
            }
        }
        len *= 2;
    }
}

pub fn ifft(a: &mut [Complex64]) {
    let n = a.len();
    for x in a.iter_mut() {
        *x = x.conj();
    }
    fft(a);
    let n_inv = 1.0 / n as f64;
    for x in a.iter_mut() {
        *x = x.conj() * n_inv;
    }
}

fn bit_reverse(a: &mut [Complex64]) {
    let n = a.len();
    let mut j = 0usize;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j |= bit;
        if i < j {
            a.swap(i, j);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn roundtrip_is_identity() {
        for n in [64, 128, 256, 512, 1024] {
            let mut a: Vec<Complex64> = (0..n)
                .map(|i| {
                    let phase = 2.0 * PI * (i as f64) / n as f64;
                    Complex64::new(phase.cos(), phase.sin())
                })
                .collect();
            let orig = a.clone();
            fft(&mut a);
            ifft(&mut a);
            for (x, y) in a.iter().zip(orig.iter()) {
                assert!((x - y).norm() < 1e-10, "n={n} diff={}", (x - y).norm());
            }
        }
    }

    #[test]
    fn convolution_theorem() {
        let n = 64;
        let a: Vec<Complex64> = (0..n)
            .map(|i| Complex64::new((i as f64).cos(), (i as f64).sin()))
            .collect();
        let b: Vec<Complex64> = (0..n)
            .map(|i| Complex64::new((i as f64 * 0.5).sin(), 0.0))
            .collect();

        let mut fa = a.clone();
        let mut fb = b.clone();
        fft(&mut fa);
        fft(&mut fb);
        for i in 0..n {
            fa[i] *= fb[i];
        }
        ifft(&mut fa);

        let mut conv = vec![Complex64::new(0.0, 0.0); n];
        for i in 0..n {
            for j in 0..n {
                conv[(i + j) % n] += a[i] * b[j];
            }
        }

        for (x, y) in fa.iter().zip(conv.iter()) {
            assert!((x - y).norm() < 1e-9);
        }
    }
}
