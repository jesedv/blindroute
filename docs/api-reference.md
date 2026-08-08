# API Reference

## Server SDK (`blindroute-server`)

### BlindRouteServer

```rust
use blindroute_server::{BlindRouteServer, CkksParams};
use blindroute_core::circuit;

let mut app = BlindRouteServer::new(CkksParams::default());
```

### Configuration

```rust
let params = CkksParams {
    n: 2048,                    // ring degree (powers of 2)
    q: 0xFFFFFFFF00000001,      // ciphertext modulus
    scale: 1 << 30,             // CKKS scaling factor
    sigma: 3.2,                 // noise standard deviation
};

let config = ServerConfig {
    host: "0.0.0.0".into(),
    port: 8080,
    cors_origins: vec!["*".into()],
    max_body_size: 1024 * 1024, // 1 MB
};

let mut app = BlindRouteServer::with_config(params, config);
```

### Key Management

```rust
// Generate new keypair
let keys = app.generate_keys();

// Export public key (to distribute to clients)
let pubkey_json = app.public_key().to_json();

// Export secret key (keep secure)
let seckey_json = app.secret_key().to_json();

// Load existing keys
app.load_keys(pubkey_json, seckey_json)?;
```

### Route Definition

Decorative style:

```rust
app.route("/v1/score", circuit! {
    inputs[0] * 0.7 + inputs[1] * 0.3
});
```

Builder style:

```rust
use blindroute_core::circuit::{Circuit, Node};

let mut c = Circuit::new();
let a = c.input(0);   // first input
let b = c.input(1);   // second input
let t1 = c.mul_const(a, 0.7);
let t2 = c.mul_const(b, 0.3);
let out = c.add(t1, t2);
c.output(out);

app.route_circuit("/v1/score", c);
```

### Supported Operations

| Operation | CKKS | BFV | Notes |
|---|---|---|---|
| `add(a, b)` | Yes | Yes | Component-wise addition |
| `sub(a, b)` | Yes | Yes | Component-wise subtraction |
| `mul(a, b)` | Yes | Yes | Tensor product + relinearize |
| `mul_const(a, c)` | Yes | Yes | Multiply by constant |
| `neg(a)` | Yes | Yes | Negation |
| `rescale(a)` | Yes | Yes | Reduce noise, consume level |

### Serving

```rust
app.serve("0.0.0.0:8080").await?;
```

### Endpoints

| Method | Path | Body | Description |
|---|---|---|---|
| GET | `/pubkey` | — | Returns public key as JSON |
| POST | `/v1/score` | Ciphertext JSON | Evaluates the registered circuit |
| GET | `/health` | — | Health check + noise budget info |
| GET | `/info` | — | Scheme info (N, Q, scale, levels) |

### Request Format

```json
{
  "scheme": "ckks",
  "inputs": [
    {
      "c0": [8234, 1192, 4401, ...],
      "c1": [7823, 3391, 2128, ...],
      "scale": 1073741824,
      "level": 4
    }
  ]
}
```

### Response Format

```json
{
  "status": "ok",
  "result": {
    "c0": [6671, 4432, 9981, ...],
    "c1": [1123, 7782, 3391, ...]
  },
  "noise_budget": {
    "remaining": 3,
    "bits": 42
  }
}
```

---

## Client SDK (`blindroute-client`)

### Native (Rust)

```rust
use blindroute_client::BlindRouteClient;
use blindroute_core::CkksParams;

let mut client = BlindRouteClient::new("https://api.example.com");
client.init().await?; // fetches public key

// Encrypt
let ct = client.encrypt_ckks(&[42.0, 73.0, 15.0])?;

// Call API
let result = client.call("/v1/score", &ct).await?;

// Decrypt
let values: Vec<f64> = client.decrypt_ckks(&result)?;
println!("{:?}", values); // → [51.3, ...]
```

### Browser (WASM)

```javascript
import { BlindRoute } from 'blindroute-wasm';

const client = new BlindRoute('https://api.example.com');
await client.init();

// Encrypt
const encrypted = await client.encrypt([42, 73, 15]);
console.log(encrypted); // { c0: [...], c1: [...], ... }

// Call API
const result = await client.call('/v1/score', encrypted);

// Decrypt
const values = client.decrypt(result);
console.log(values); // → [51.3, ...]
```

### WASM Browser API

| Method | Description |
|---|---|
| `new BlindRoute(baseUrl)` | Create client pointing to server |
| `await client.init()` | Fetch public key from server |
| `await client.encrypt(numbers)` | Encrypt array of numbers |
| `client.decrypt(ciphertext)` | Decrypt ciphertext to numbers |
| `await client.call(path, ct)` | Full roundtrip: encrypt → send → decrypt |
| `client.schemeInfo()` | Get scheme parameters |
| `client.runSelfTest()` | Run 601 NTT checks in browser |
| `client.runCkksSelfTest()` | Run CKKS roundtrip check |

---

## CLI

```bash
blindroute keygen --out <dir>
blindroute encrypt --pub <pk> --in <data> --out <ct>
blindroute compute add <a> <b> --out <r>
blindroute compute mul <a> <b> --out <r>
blindroute compute sum <a> <b> [c...] --out <r>
blindroute decrypt --sec <sk> --in <ct>
blindroute serve --port <port>
blindroute bench
blindroute help
```

### Example Session

```bash
# Generate keys
$ blindroute keygen --out keys/
Generated keypair: keys/pub.json, keys/sec.json

# Create input data
$ echo '[42, 73, 15, 88]' > data.json

# Encrypt
$ blindroute encrypt --pub keys/pub.json --in data.json --out ct.json
Encrypted 4 values

# Homomorphic add (server computes without seeing numbers)
$ blindroute compute add ct.json ct.json --out sum.json
Computed: Add(ct, ct) → sum.json

# Decrypt
$ blindroute decrypt --sec keys/sec.json --in sum.json
[84.0, 146.0, 30.0, 176.0]
```

---

## Configuration Reference

### CkksParams

| Field | Default | Description |
|---|---|---|
| `n` | 2048 | Ring degree (power of 2) |
| `q` | `0xFFFFFFFF00000001` | Ciphertext modulus (64-bit) |
| `scale` | `1 << 30` | CKKS scaling factor |
| `sigma` | 3.2 | Gaussian noise standard deviation |
| `levels` | 8 | Maximum multiplicative depth |

### BfvParams

| Field | Default | Description |
|---|---|---|
| `n` | 2048 | Ring degree |
| `q` | Chain of moduli | Ciphertext modulus chain |
| `t` | 65537 | Plaintext modulus |
| `sigma` | 3.2 | Noise standard deviation |
| `levels` | 8 | Maximum depth |
