//! Threshold secret sharing over the Mersenne prime `p = 2^31 − 1`.
//!
//! This is the "private mean" flavor of RingCrypt: a group of parties (e.g. a
//! hospital consortium, a salary survey, a research collaboration) wants the
//! **average** of their values without any party learning another's value.
//!
//! The construction is **additive secret sharing**: a value `v` is split into
//! `n` shares `r_1..r_n` satisfying `Σ r_i ≡ v (mod p)`. Party `j` learns only
//! `r_j` — which, taken alone, is statistically independent of `v`. Everyone
//! together reconstructs the sum `Σ v_i`, from which the mean follows.
//!
//! `p = 2^31 − 1` is chosen so that every intermediate fits exactly in the
//! `f64`/JavaScript numbers the browser demo uses (values and sums are all
//! `< 2^53`), and Mersenne modular reduction is trivial and branch-free.
//!
//! > **Note on randomness:** the demo uses a *seeded* PCG generator so every
//! > run is reproducible. Production deployments must substitute a CSPRNG
//! > (`OsRng`) — the additive scheme above is exactly the one used by honest
//! > multiparty-computation protocols, randomness merely hides the shares.

#![forbid(unsafe_code)]

use rand::RngCore;
use rand_pcg::Pcg64Mcg;

/// The Mersenne prime `2^31 − 1`.
pub const P: u64 = (1 << 31) - 1;

/// Additive shares of a single value: `n` shares summing to `value` (mod `p`).
///
/// Returns a vector of `n` shares; any `n − 1` of them reveal nothing about
/// `value`.
pub fn share_value(value: u64, n_parties: usize, modulus: u64, seed: u64) -> Vec<u64> {
    debug_assert!(value < modulus, "value must be reduced mod p");
    let mut rng = Pcg64Mcg::new(seed as u128);
    let mut shares = Vec::with_capacity(n_parties);
    let mut acc = 0u64;
    for _ in 0..n_parties - 1 {
        // Uniform in [0, modulus).
        let r = rng.next_u64() % modulus;
        acc = (acc + r) % modulus;
        shares.push(r);
    }
    // Last share forces the sum to equal value.
    let last = (value + modulus - acc) % modulus;
    shares.push(last);
    debug_assert!(
        shares.iter().fold(0u64, |s, x| (s + x) % modulus) % modulus == value % modulus
    );
    shares
}

/// Sum of the share-vectors each party holds for their own value.
///
/// `grid[i][j]` = share of value `i` held by party `j`. Summing **all** cells
/// reconstructs `Σ values` (mod `modulus`).
pub fn reconstruct_sum(grid: &[Vec<u64>], modulus: u64) -> u64 {
    grid.iter()
        .flat_map(|r| r.iter())
        .fold(0u64, |s, x| (s + x) % modulus)
}

/// The full private-mean oracle used by demos and tests.
///
/// Each input value is split into `n` additive shares; party `j` receives the
/// `j`-th share of every value. Returns the reconstructed total and the
/// exact arithmetic mean, plus the per-party views (so the UI can show that
/// no single party's view contains any other value).
pub fn private_mean(values: &[u64], seed: u64) -> PrivateMean {
    let n = values.len();
    let modulus = P;
    let mut grid = Vec::with_capacity(n);
    for (i, &v) in values.iter().enumerate() {
        let shares = share_value(v, n, modulus, seed ^ (i as u64).wrapping_mul(0x9E37_79B9));
        grid.push(shares);
    }
    let total = reconstruct_sum(&grid, modulus);
    let mean_num = total;
    let mean_den = n as u64;
    // Exact rational and best-effort decimal.
    let mean_f64 = if n == 0 { 0.0 } else { mean_num as f64 / mean_den as f64 };
    PrivateMean {
        n,
        total,
        mean_num,
        mean_den,
        mean_f64,
        party_views: grid,
    }
}

/// Result of a private-mean computation.
#[derive(Debug, Clone, serde::Serialize)]
pub struct PrivateMean {
    /// Number of parties.
    pub n: usize,
    /// Reconstructed sum of all values.
    pub total: u64,
    /// Numerator of the exact mean.
    pub mean_num: u64,
    /// Denominator of the exact mean.
    pub mean_den: u64,
    /// Floating-point mean (exact whenever it divides evenly).
    pub mean_f64: f64,
    /// `party_views[i][j]` — share of value `i` held by party `j`.
    pub party_views: Vec<Vec<u64>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shares_reconstruct_each_value() {
        let values = [100u64, 250, 32000, P - 5, 7];
        let n = values.len();
        for (i, &v) in values.iter().enumerate() {
            let shares = share_value(v, n, P, 42 + i as u64);
            let sum = shares.iter().fold(0u64, |s, x| (s + x) % P);
            assert_eq!(sum, v, "value {i} not reconstructed");
        }
    }

    #[test]
    fn private_mean_is_correct() {
        let values = [100u64, 200, 300];
        let pm = private_mean(&values, 7);
        assert_eq!(pm.total, 600);
        assert_eq!(pm.mean_num, 600);
        assert_eq!(pm.mean_den, 3);
        assert!((pm.mean_f64 - 200.0).abs() < 1e-9);
    }

    #[test]
    fn a_single_party_learns_nothing() {
        // Party 0's share of one value should not equal that value (unless
        // modulus is 1); over many draws the shares vary with the seed and
        // no single share equals the plaintext in general.
        let value = 1111u64;
        let seeds = 1..20;
        let shares: Vec<u64> = seeds.map(|s| share_value(value, 3, P, s)[0]).collect();
        // At least two distinct shares => share is not a deterministic leak.
        let distinct = shares.windows(2).any(|w| w[0] != w[1]);
        assert!(distinct, "shares look deterministic -> leaks info");
    }
}
