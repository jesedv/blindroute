use crate::params::BfvParams;

pub fn encode(params: &BfvParams, messages: &[i64]) -> Vec<u64> {
    let n = params.n;
    let q = params.q;
    let t = params.t;

    let slots = n / 2;
    let mut poly = vec![0u64; n];

    for (i, &m) in messages.iter().enumerate() {
        if i >= slots {
            break;
        }
        let m_mod = (m % (t as i64) + (t as i64)) % (t as i64);
        let encoded = (m_mod as u128) * (params.delta as u128) % (q as u128);
        poly[i] = encoded as u64;
    }

    poly
}

pub fn decode(params: &BfvParams, poly: &[u64], scale_power: usize) -> Vec<i64> {
    let n = params.n;
    let q = params.q;
    let t = params.t;
    let half_q = q / 2;

    let slots = n / 2;
    let mut messages = Vec::with_capacity(slots);

    for i in 0..slots {
        let mut val = poly[i] as u128;
        for _ in 0..scale_power {
            val = (val * (t as u128) + (half_q as u128)) / (q as u128);
        }
        let m = (val % (t as u128)) as i64;
        messages.push(m);
    }

    messages
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::BfvParams;

    #[test]
    fn encode_decode_roundtrip() {
        let params = BfvParams::default();
        let msgs = vec![42i64, 73, 15, 88, 100, 200, 300, 400];
        let encoded = encode(&params, &msgs);
        let decoded = decode(&params, &encoded, 1);
        assert_eq!(&decoded[..msgs.len()], &msgs[..]);
    }

    #[test]
    fn negative_values() {
        let params = BfvParams::default();
        let msgs = vec![-5i64, -10, -100];
        let encoded = encode(&params, &msgs);
        let decoded = decode(&params, &encoded, 1);
        let t = params.t as i64;
        assert_eq!(decoded[0], (-5i64).rem_euclid(t));
        assert_eq!(decoded[1], (-10i64).rem_euclid(t));
        assert_eq!(decoded[2], (-100i64).rem_euclid(t));
    }
}
