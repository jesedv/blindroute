pub const NTT_SHADER: &str = r#"
struct Uniforms {
    n: u32,
    len: u32,
    q: u32,
    _pad: u32,
}

@group(0) @binding(0) var<storage, read_write> data: array<u32>;
@group(0) @binding(1) var<storage, read> w_powers: array<u32>;
@group(0) @binding(2) var<uniform> u: Uniforms;

@compute @workgroup_size(256)
fn ntt_stage(@builtin(global_invocation_id) gid: vec3<u32>) {
    let total_pairs = u.n / 2u;
    if (gid.x >= total_pairs) {
        return;
    }

    let len = u.len;
    let group_size = 2u * len;
    let group = gid.x / len;
    let offset = gid.x % len;
    let start = group * group_size;

    let i = start + offset;
    let j = i + len;
    let w = w_powers[offset];

    let ui = data[i];
    let uj = (data[j] * w) % u.q;

    data[i] = (ui + uj) % u.q;
    data[j] = (ui + u.q - uj) % u.q;
}
"#;
