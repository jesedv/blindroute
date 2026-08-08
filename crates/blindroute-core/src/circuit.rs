use serde::{Deserialize, Serialize};

use crate::scheme::FheScheme;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Node {
    Input(usize),
    ConstF64(f64),
    Add(usize, usize),
    Sub(usize, usize),
    Mul(usize, usize),
    Neg(usize),
    Rescale(usize),
}

impl Node {
    pub fn dependencies(&self) -> Vec<usize> {
        match self {
            Node::Input(_) | Node::ConstF64(_) => vec![],
            Node::Add(a, b) | Node::Sub(a, b) | Node::Mul(a, b) => vec![*a, *b],
            Node::Neg(a) | Node::Rescale(a) => vec![*a],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Circuit {
    pub nodes: Vec<Node>,
    pub outputs: Vec<usize>,
    pub num_inputs: usize,
}

impl Circuit {
    pub fn new(num_inputs: usize) -> Self {
        let mut nodes = Vec::with_capacity(num_inputs);
        for i in 0..num_inputs {
            nodes.push(Node::Input(i));
        }
        Circuit {
            nodes,
            outputs: Vec::new(),
            num_inputs,
        }
    }

    pub fn input_idx(&self, idx: usize) -> usize {
        assert!(idx < self.num_inputs, "input index out of bounds");
        idx
    }

    pub fn const_f64(&mut self, value: f64) -> usize {
        let idx = self.nodes.len();
        self.nodes.push(Node::ConstF64(value));
        idx
    }

    pub fn add(&mut self, a: usize, b: usize) -> usize {
        let idx = self.nodes.len();
        self.nodes.push(Node::Add(a, b));
        idx
    }

    pub fn sub(&mut self, a: usize, b: usize) -> usize {
        let idx = self.nodes.len();
        self.nodes.push(Node::Sub(a, b));
        idx
    }

    pub fn mul(&mut self, a: usize, b: usize) -> usize {
        let idx = self.nodes.len();
        self.nodes.push(Node::Mul(a, b));
        idx
    }

    pub fn neg(&mut self, a: usize) -> usize {
        let idx = self.nodes.len();
        self.nodes.push(Node::Neg(a));
        idx
    }

    pub fn rescale(&mut self, a: usize) -> usize {
        let idx = self.nodes.len();
        self.nodes.push(Node::Rescale(a));
        idx
    }

    pub fn output(&mut self, node_idx: usize) {
        self.outputs.push(node_idx);
    }

    pub fn evaluate<S: FheScheme>(
        &self,
        inputs: &[S::Ciphertext],
        ek: &S::EvaluationKey,
        params: &S::Params,
        pk: Option<&S::PublicKey>,
    ) -> Result<Vec<S::Ciphertext>, String> {
        if inputs.len() != self.num_inputs {
            return Err(format!(
                "expected {} inputs, got {}",
                self.num_inputs,
                inputs.len()
            ));
        }

        let mut wire: Vec<Option<S::Ciphertext>> = vec![None; self.nodes.len()];

        for (i, node) in self.nodes.iter().enumerate() {
            let ct = match node {
                Node::Input(idx) => inputs[*idx].clone(),
                Node::ConstF64(val) => {
                    let pk = pk.ok_or("ConstF64 requires pk for encryption")?;
                    let pt = S::encode(params, &[*val]);
                    S::encrypt(pk, &pt)
                }
                Node::Add(a, b) => {
                    let a_ct = wire[*a].as_ref().ok_or("missing wire value for add")?;
                    let b_ct = wire[*b].as_ref().ok_or("missing wire value for add")?;
                    S::add(ek, a_ct, b_ct)
                }
                Node::Sub(a, b) => {
                    let a_ct = wire[*a].as_ref().ok_or("missing wire value for sub")?;
                    let b_ct = wire[*b].as_ref().ok_or("missing wire value for sub")?;
                    S::sub(ek, a_ct, b_ct)
                }
                Node::Mul(a, b) => {
                    let a_ct = wire[*a].as_ref().ok_or("missing wire value for mul")?;
                    let b_ct = wire[*b].as_ref().ok_or("missing wire value for mul")?;
                    S::multiply(ek, a_ct, b_ct)
                }
                Node::Neg(a) => {
                    let a_ct = wire[*a].as_ref().ok_or("missing wire value for neg")?;
                    S::negate(ek, a_ct)
                }
                Node::Rescale(a) => {
                    let a_ct = wire[*a].as_ref().ok_or("missing wire value for rescale")?;
                    S::rescale(ek, a_ct).ok_or("rescale exhausted modulus chain")?
                }
            };
            wire[i] = Some(ct);
        }

        let mut results = Vec::with_capacity(self.outputs.len());
        for &out_idx in &self.outputs {
            let ct = wire[out_idx]
                .clone()
                .ok_or(format!("missing wire value for output {}", out_idx))?;
            results.push(ct);
        }
        Ok(results)
    }

    pub fn multiplicative_depth(&self) -> usize {
        let mut depths = vec![0usize; self.nodes.len()];
        for (i, node) in self.nodes.iter().enumerate() {
            depths[i] = match node {
                Node::Input(_) | Node::ConstF64(_) => 0,
                Node::Add(a, b) | Node::Sub(a, b) => depths[*a].max(depths[*b]),
                Node::Mul(a, b) => depths[*a].max(depths[*b]) + 1,
                Node::Neg(a) => depths[*a],
                Node::Rescale(a) => depths[*a].saturating_sub(1),
            };
        }
        if self.outputs.is_empty() {
            return depths.iter().copied().max().unwrap_or(0);
        }
        self.outputs.iter().map(|&idx| depths[idx]).max().unwrap_or(0)
    }

    pub fn op_count(&self) -> usize {
        self.nodes
            .iter()
            .filter(|n| {
                !matches!(n, Node::Input(_) | Node::ConstF64(_))
            })
            .count()
    }

    pub fn linear_combination(&mut self, inputs: &[usize], weights: &[f64]) -> usize {
        assert_eq!(inputs.len(), weights.len(), "inputs and weights must match");
        let mut sum = self.const_f64(0.0);
        for (i, (&inp, &w)) in inputs.iter().zip(weights.iter()).enumerate() {
            let w_node = self.const_f64(w);
            let term = self.mul(inp, w_node);
            sum = self.add(sum, term);
        }
        sum
    }

    pub fn sigmoid_approx_3(&mut self, x: usize) -> usize {
        let half = self.const_f64(0.5);
        let quarter = self.const_f64(0.25);
        let x_sq = self.mul(x, x);
        let x_cu = self.mul(x_sq, x);
        let t1 = self.mul(x, quarter);
        let t2_coeff = self.const_f64(1.0 / 48.0);
        let t2 = self.mul(x_cu, t2_coeff);
        let term = self.sub(t1, t2);
        self.add(half, term)
    }

    pub fn sigmoid_approx_5(&mut self, x: usize) -> usize {
        let half = self.const_f64(0.5);
        let c1 = self.const_f64(0.25);
        let c3 = self.const_f64(1.0 / 48.0);
        let c5 = self.const_f64(1.0 / 480.0);
        let x2 = self.mul(x, x);
        let x3 = self.mul(x2, x);
        let x4 = self.mul(x3, x);
        let x5 = self.mul(x4, x);
        let t1 = self.mul(x, c1);
        let t3 = self.mul(x3, c3);
        let t5 = self.mul(x5, c5);
        let sum = self.add(t1, t5);
        let sum = self.sub(sum, t3);
        self.add(half, sum)
    }

    pub fn relu_approx_square(&mut self, x: usize) -> usize {
        self.mul(x, x)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_circuit() {
        let c = Circuit::new(2);
        assert_eq!(c.num_inputs, 2);
        assert_eq!(c.nodes.len(), 2);
        assert_eq!(c.multiplicative_depth(), 0);
        assert_eq!(c.op_count(), 0);
    }

    #[test]
    fn simple_add_circuit() {
        let mut c = Circuit::new(2);
        let out = c.add(0, 1);
        c.output(out);

        assert_eq!(c.nodes.len(), 3);
        assert_eq!(c.multiplicative_depth(), 0);
        assert_eq!(c.op_count(), 1);
    }

    #[test]
    fn nested_mul_circuit() {
        let mut c = Circuit::new(2);
        let t1 = c.mul(0, 1);
        let t2 = c.mul(t1, 0);
        c.output(t2);

        assert_eq!(c.multiplicative_depth(), 2);
    }

    #[test]
    fn mixed_depth() {
        let mut c = Circuit::new(3);
        let add = c.add(0, 1);
        let mul = c.mul(add, 2);
        let sub = c.sub(mul, 0);
        c.output(sub);

        assert_eq!(c.multiplicative_depth(), 1);
        assert_eq!(c.op_count(), 3);
    }

    #[test]
    fn rescale_reduces_depth() {
        let mut c = Circuit::new(2);
        let mul = c.mul(0, 1);
        let rescale = c.rescale(mul);
        c.output(rescale);

        assert_eq!(c.multiplicative_depth(), 0);
    }

    #[test]
    fn dependencies() {
        let mut c = Circuit::new(1);
        let cst = c.const_f64(3.0);
        let add = c.add(0, cst);

        assert!(matches!(c.nodes[cst], Node::ConstF64(3.0)));
        assert_eq!(c.nodes[add].dependencies(), vec![0, cst]);
    }

    #[test]
    fn sigmoid_approx_3_depth() {
        let mut c = Circuit::new(1);
        let out = c.sigmoid_approx_3(0);
        c.output(out);
        assert!(c.multiplicative_depth() >= 2);
        assert!(c.op_count() >= 4);
    }

    #[test]
    fn sigmoid_approx_5_depth() {
        let mut c = Circuit::new(1);
        let out = c.sigmoid_approx_5(0);
        c.output(out);
        assert!(c.multiplicative_depth() >= 3);
        assert!(c.op_count() >= 6);
    }

    #[test]
    fn linear_combination() {
        let mut c = Circuit::new(3);
        let out = c.linear_combination(&[0, 1, 2], &[0.6, 0.3, 0.1]);
        c.output(out);
        assert!(c.op_count() >= 5);
    }
}
