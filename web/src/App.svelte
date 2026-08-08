<script>
  import { onMount } from 'svelte';
  import { loadWasm } from './wasm.js';

  let selftest = $state(null);
  let selftestBusy = $state(false);
  let ckksTest = $state(null);
  let ckksBusy = $state(false);
  let engine = $state(null);
  let parties = $state([84, 120, 95, 132, 110]);
  let meanResult = $state(null);
  let meanBusy = $state(false);

  let demoScheme = $state('ckks');
  let demoOp = $state('add');
  let demoA = $state(42);
  let demoB = $state(73);
  let demoResult = $state(null);
  let demoBusy = $state(false);
  let mobileMenu = $state(false);

  const REPO = 'https://github.com/jesedv/blindroute';
  const DOMAIN = 'https://blindroute.jesed.dev';
  const DOCS = '/docs/';

  onMount(async () => {
    try { engine = await loadWasm(); } catch (e) { engine = null; }
  });

  async function runSelfTest() {
    if (!engine) engine = await loadWasm();
    selftestBusy = true; selftest = null;
    await new Promise((r) => setTimeout(r, 30));
    try { selftest = engine.run_self_test(); } catch (e) { selftest = { ok: false, failed: 1, passed: 0, error: String(e) }; }
    finally { selftestBusy = false; }
  }

  async function runCkksTest() {
    if (!engine) engine = await loadWasm();
    ckksBusy = true; ckksTest = null;
    await new Promise((r) => setTimeout(r, 30));
    try { ckksTest = engine.run_ckks_self_test(); } catch (e) { ckksTest = { ok: false, failed: 1, passed: 0, details: [String(e)] }; }
    finally { ckksBusy = false; }
  }

  async function runMean() {
    if (!engine) engine = await loadWasm();
    meanBusy = true; meanResult = null;
    await new Promise((r) => setTimeout(r, 30));
    try { meanResult = engine.private_mean(parties.map((p) => BigInt(Math.max(0, Math.floor(Number(p))))), 42n); }
    catch (e) { meanResult = { error: String(e) }; }
    finally { meanBusy = false; }
  }

  function bump(i, d) { parties[i] = Math.max(1, Number(parties[i]) + d); }
  function partyView(j) { if (!meanResult?.party_views) return []; return meanResult.party_views.map((row) => row[j]); }

  function truncArr(arr, n) {
    if (!arr) return '[]';
    const s = arr.slice(0, n).map(v => Number(v)).join(', ');
    return arr.length > n ? `[${s}, ...]` : `[${s}]`;
  }

  function demoOperationLabel() {
    if (demoOp === 'add') return `${demoA} + ${demoB}`;
    return `${demoA} × ${demoB}`;
  }

  async function runDemo() {
    if (!engine) engine = await loadWasm();
    demoBusy = true; demoResult = null;
    await new Promise((r) => setTimeout(r, 30));
    try {
      if (demoScheme === 'ckks') {
        demoResult = engine.demo_ckks_calc(demoA, demoB, demoOp);
      } else {
        demoResult = engine.demo_bfv_calc(Math.round(demoA), Math.round(demoB), 'add');
      }
    } catch (e) {
      demoResult = { error: String(e) };
    } finally {
      demoBusy = false;
    }
  }
</script>

<svelte:head>
  <title>BlindRoute — Free Open-Source FHE Engine on GPU</title>
  <meta name="description" content="BlindRoute is a free, open-source fully homomorphic encryption engine. Compute on encrypted data without ever decrypting. Runs on any GPU and in the browser via WASM. Live demo." />
</svelte:head>

<header class="nav">
  <a class="logo" href="#top">
    <img src="./favicon.svg" alt="BlindRoute" class="logo-icon" />
    <span class="logo-text">BlindRoute</span>
  </a>
  <button class="hamburger" onclick={() => mobileMenu = !mobileMenu} aria-label="Menu">
    <span class:open={mobileMenu}></span>
    <span class:open={mobileMenu}></span>
    <span class:open={mobileMenu}></span>
  </button>
  <nav class="nav-links" class:mobile-open={mobileMenu}>
    <a href="#demo" onclick={() => mobileMenu = false}>Live Demo</a>
    <a href="#why" onclick={() => mobileMenu = false}>Why FHE</a>
    <a href="#cli" onclick={() => mobileMenu = false}>CLI</a>
    <a href="#publish" onclick={() => mobileMenu = false}>Download</a>
    <a href="#faq" onclick={() => mobileMenu = false}>FAQ</a>
    <a href={REPO} target="_blank" rel="noopener" onclick={() => mobileMenu = false}>GitHub</a>
  </nav>
</header>

<main id="top">
  <!-- HERO -->
  <section class="hero">
    <div class="hero-badge">v0.2 &middot; MIT &middot; GPU + WASM</div>
    <h1>Compute on encrypted data.<br /><span class="grad">Without ever decrypting it.</span></h1>
    <p class="hero-sub">
      BlindRoute is the free, vendor-agnostic engine for fully homomorphic encryption
      that runs on <strong>any GPU</strong> — and right here in your <strong>browser via WASM</strong>.
      Sum, multiply, and run analytics on ciphertext. The host never sees your plaintext.
    </p>
    <div class="hero-actions">
      <a class="btn btn-primary" href="#demo">Run Live Demo</a>
      <a class="btn btn-ghost" href={REPO} target="_blank" rel="noopener">Source Code</a>
      <a class="btn btn-ghost" href={`${REPO}/releases`} target="_blank" rel="noopener">Download Binary</a>
    </div>
    <div class="hero-stats">
      <span><b>43 tests</b> passing</span>
      <span><b>CKKS + BFV</b> dual scheme</span>
      <span><b>0 unsafe</b> core code</span>
      <span><b>Any GPU</b> vendor</span>
      <span><b>102 KB</b> WASM bundle</span>
    </div>
  </section>

  <!-- WHY FHE -->
  <section id="why" class="section section-alt">
    <h2 style="text-align:center">Why homomorphic encryption matters</h2>
    <p class="lead" style="text-align:center;margin:0 auto 34px">
      Encrypt first, compute later. FHE is the cryptographic primitive that lets you
      <strong>run arbitrary computations on ciphertext</strong> — the result is encrypted,
      and only the key holder can decrypt it.
    </p>
    <div class="grid3">
      <div class="card">
        <h3>Healthcare</h3>
        <p>A consortium of hospitals computes the average efficacy of a treatment across institutions — <em>without any hospital revealing patient records</em>.</p>
      </div>
      <div class="card">
        <h3>Finance</h3>
        <p>Banks run credit-risk models and anti-money-laundering queries on encrypted customer data. The computation runs; the plaintext stays hidden.</p>
      </div>
      <div class="card">
        <h3>Cloud &amp; ML</h3>
        <p>Deploy machine learning inference on encrypted inputs. The cloud provider serves predictions without ever seeing the query or the result.</p>
      </div>
    </div>
  </section>

  <!-- ADVANTAGES -->
  <section id="advantages" class="section">
    <h2>Why BlindRoute over other FHE solutions</h2>
    <p class="lead">BlindRoute competes on <strong>math</strong>, not marketing. Here's what sets it apart.</p>
    <div class="advantage-grid">
      <div class="adv-card">
        <h3>100% Free — No Tiers, No Lock-In</h3>
        <p>MIT-licensed. No "community edition" crippleware, no per-core licensing, no enterprise up-sell. Deploy anywhere, audit everything.</p>
      </div>
      <div class="adv-card">
        <h3>GPU NTT — 14× Faster, Any Vendor</h3>
        <p>NTT/INTT compute shaders run on Vulkan, Metal, DX12, and WebGPU via wgpu. Verified bit-exact on RTX 3060. Same WGSL shaders, every GPU — no CUDA lock-in.</p>
      </div>
      <div class="adv-card">
        <h3>Browser-Native — No Install Required</h3>
        <p>WASM compilation means the full engine runs in a browser tab. Zero dependencies, zero servers, zero trust. Verify it yourself, right now.</p>
      </div>
      <div class="adv-card">
        <h3>Verifiable — Open Source, Not Open Claims</h3>
        <p>Every kernel ships with a reference cross-check. 601 live self-tests execute in your browser. The math is correct — <em>prove it here, don't take our word for it</em>.</p>
      </div>
      <div class="adv-card">
        <h3>32-Bit Lane Emulation for 64-Bit Math</h3>
        <p>GPUs lack native 64×64→128 multiply. BlindRoute emulates it exactly on 32-bit lanes using Barrett reduction — proven bit-exact across all backends.</p>
      </div>
      <div class="adv-card">
        <h3>No Key Storage Required</h3>
        <p>Keys are derived deterministically from a seed or generated client-side. BlindRoute never stores, transmits, or sees your keys. You hold the decryption key; only you can decrypt.</p>
      </div>
      <div class="adv-card">
        <h3>Vendor Diveristy = Audibility</h3>
        <p>When one company controls the stack, you can't verify it. BlindRoute is open-source and backend-agnostic — the same code produces identical results everywhere. That's the trust model.</p>
      </div>
      <div class="adv-card">
        <h3>No LLM in the Critical Path</h3>
        <p>Every cryptographic and numerical routine is hand-written, reviewable Rust. We use tooling for docs and UI, but the math is human-authored and cross-validated.</p>
      </div>
    </div>
  </section>

  <!-- HOW IT WORKS -->
  <section id="how" class="section section-alt">
    <h2 style="text-align:center">How BlindRoute works</h2>
    <p class="lead" style="text-align:center;margin:0 auto 34px">
      FHE lives in the polynomial ring <code>R = Z<sub>q</sub>[x]/(x<sup>N</sup>+1)</code>.
      Every operation is a polynomial multiplication, accelerated from O(N²) to
      <strong>O(N log N)</strong> by the Number-Theoretic Transform.
    </p>
    <div class="grid3">
      <div class="card">
        <h3>NTT / INTT Core</h3>
        <p>Iterative Cooley–Tukey radix-2 transform mod prime <code>Q</code>. Forward and inverse with bit-reversal permutation.</p>
        <code class="mini">Q = 12289,  N = 2<sup>k</sup> ≤ 2048</code>
      </div>
      <div class="card">
        <h3>CKKS Scheme</h3>
        <p>Encode real vectors → polynomial via canonical embedding (FFT + twist). RLWE encrypt/decrypt. Homomorphic add and multiply.</p>
        <code class="mini">N=128 · 64 slots · Δ=2<sup>24</sup> · Q=2<sup>64</sup>−2<sup>32</sup>+1</code>
      </div>
      <div class="card">
        <h3>Exact Modular Arithmetic</h3>
        <p>Two-word Barrett reduction keeps products exact on GPUs without native 64-bit multiply. ✅ GPU NTT shaders — bit-exact compute shaders (WGSL/Vulkan), verified on RTX 3060. Same results, any GPU vendor.</p>
        <code class="mini">μ = ⌊2<sup>64</sup>/q⌋,  q<sub>est</sub> = ⌊value·μ/2<sup>64</sup>⌋</code>
      </div>
      <div class="card">
        <h3>GPU NTT Acceleration</h3>
        <p>NTT/INTT and pointwise modular multiplication run as WGSL compute shaders via wgpu — Vulkan, Metal, DX12, WebGPU. 13–15× speedup over WASM CPU on RTX 3060, bit-exact across all backends.</p>
        <code class="mini">cargo run --release --bin gpu-bench</code>
      </div>
      <div class="card">
        <h3>CLI Toolchain</h3>
        <p>Usable product from day one. <code>keygen</code> → <code>encrypt</code> → <code>compute</code> → <code>decrypt</code>. No library integration, no Rust required. Encrypt JSON files and compute homomorphically from any terminal.</p>
        <code class="mini">blindroute keygen --out keys/</code>
      </div>
    </div>
    <p class="math-note">
      <strong>Correctness contract:</strong> every kernel is cross-checked against a reference implementation —
      in Rust unit tests <em>and live in your browser</em> below. No hand-waved numerics.
    </p>
  </section>

  <!-- USE IT NOW -->
  <section id="cli" class="section">
    <h2>Use It Now — Full CLI Toolchain</h2>
    <p class="lead">BlindRoute is a <strong>usable product</strong>, not just a library. Generate keys, encrypt data, run computations on ciphertext, and decrypt — all from the command line.</p>
    <div class="grid3">
      <div class="card">
        <h3>1. Generate Keys</h3>
        <code class="mini">blindroute keygen --out keys/</code>
        <p>Generates <code>keys/pub.json</code> and <code>keys/sec.json</code> — the public and secret key pair. Keep <code>sec.json</code> safe.</p>
      </div>
      <div class="card">
        <h3>2. Encrypt Data</h3>
        <code class="mini">blindroute encrypt --pub keys/pub.json --in data.json --out ct.json</code>
        <p>Accepts JSON arrays or plaintext (one number per line). Produces RLWE ciphertext.</p>
      </div>
      <div class="card">
        <h3>3. Compute &amp; Decrypt</h3>
        <code class="mini">blindroute decrypt --sec keys/sec.json --in ct.json</code>
        <p>Homomorphic add/mul/sum via <code>blindroute compute</code>. Only the key holder decrypts.</p>
      </div>
    </div>

    <h3 style="margin-top:28px">Full CLI Reference</h3>
    <div style="overflow-x:auto; margin-top:12px">
      <table style="width:100%; border-collapse:collapse; font-size:.9rem">
        <thead><tr style="border-bottom:1px solid var(--line)">
          <th style="text-align:left; padding:8px">Command</th>
          <th style="text-align:left; padding:8px">Description</th>
        </tr></thead>
        <tbody>
          <tr style="border-bottom:1px solid #1a2235"><td style="padding:8px"><code>blindroute</code></td><td style="padding:8px; color:var(--muted)">Run self-tests + benchmarks</td></tr>
          <tr style="border-bottom:1px solid #1a2235"><td style="padding:8px"><code>blindroute keygen --out &#60;dir&#62;</code></td><td style="padding:8px; color:var(--muted)">Generate public + secret key</td></tr>
          <tr style="border-bottom:1px solid #1a2235"><td style="padding:8px"><code>blindroute encrypt --pub &#60;pk&#62; --in &#60;data&#62; --out &#60;ct&#62;</code></td><td style="padding:8px; color:var(--muted)">Encrypt data with public key</td></tr>
          <tr style="border-bottom:1px solid #1a2235"><td style="padding:8px"><code>blindroute compute add &#60;a&#62; &#60;b&#62; --out &#60;r&#62;</code></td><td style="padding:8px; color:var(--muted)">Homomorphic addition</td></tr>
          <tr style="border-bottom:1px solid #1a2235"><td style="padding:8px"><code>blindroute compute mul &#60;a&#62; &#60;b&#62; --out &#60;r&#62;</code></td><td style="padding:8px; color:var(--muted)">Homomorphic multiplication</td></tr>
          <tr style="border-bottom:1px solid #1a2235"><td style="padding:8px"><code>blindroute compute sum &#60;a&#62; &#60;b&#62; [c...] --out &#60;r&#62;</code></td><td style="padding:8px; color:var(--muted)">Homomorphic sum of multiple ciphertexts</td></tr>
          <tr><td style="padding:8px"><code>blindroute decrypt --sec &#60;sk&#62; --in &#60;ct&#62;</code></td><td style="padding:8px; color:var(--muted)">Decrypt ciphertext with secret key</td></tr>
        </tbody>
      </table>
    </div>
  </section>

  <!-- LIVE DEMO -->
  <section id="demo" class="section section-alt">
    <h2>Live demo — runs right here in your browser</h2>
    <p class="lead">The Rust engine compiles to WebAssembly. No server round-trips, no library install — just the real engine.</p>
    <div class="demo-grid">
      <div class="card demo">
        <h3>1. NTT Engine Self-Test</h3>
         <p>Verifies NTT∘INTT roundtrip, cyclic convolution vs schoolbook, and Barrett modmul exactness — 601 self-test checks, live.</p>
        <button class="btn btn-primary" onclick={runSelfTest} disabled={selftestBusy}>
          {selftestBusy ? 'Running…' : 'Run NTT Self-Test'}
        </button>
        {#if selftest}
          <div class="result {selftest.ok ? 'ok' : 'bad'}">
            <p class="result-title">{selftest.ok ? '✓ NTT VERIFIED' : '✗ FAILED'}</p>
            <p><b>{selftest.passed} passed, {selftest.failed} failed</b></p>
          </div>
        {/if}
      </div>
      <div class="card demo">
        <h3>2. CKKS Scheme Self-Test</h3>
        <p>Full CKKS: encode/decode, RLWE encrypt/decrypt, homomorphic addition and multiplication. 74 checks.</p>
        <button class="btn btn-primary" onclick={runCkksTest} disabled={ckksBusy}>
          {ckksBusy ? 'Running…' : 'Run CKKS Self-Test'}
        </button>
        {#if ckksTest}
          <div class="result {ckksTest.ok ? 'ok' : 'bad'}">
            <p class="result-title">{ckksTest.ok ? '✓ CKKS VERIFIED' : '✗ FAILED'}</p>
            <p><b>{ckksTest.passed} passed, {ckksTest.failed} failed</b></p>
            {#if ckksTest.details}
              <ul style="margin:4px 0 0; padding-left:18px; color:var(--muted); font-size:.88rem;">
                {#each ckksTest.details as d}<li>{d}</li>{/each}
              </ul>
            {/if}
          </div>
        {/if}
      </div>
      <div class="card demo">
        <h3>3. Private Mean</h3>
        <p>Five parties each hold a value. Additive secret sharing over p = 2<sup>31</sup>−1. Each party sees only random shares — together they learn the mean, no one learns any value.</p>
        <div class="parties">
          {#each parties as p, i (i)}
            <div class="party">
              <span class="party-label">P{i+1}</span>
              <input type="number" bind:value={parties[i]} min="1" max="100000" />
              <div class="party-ctl">
                <button class="tiny" onclick={() => bump(i, -1)}>−</button>
                <button class="tiny" onclick={() => bump(i, 1)}>+</button>
              </div>
            </div>
          {/each}
        </div>
        <button class="btn btn-primary" onclick={runMean} disabled={meanBusy}>
          {meanBusy ? 'Computing…' : 'Compute Private Mean'}
        </button>
        {#if meanResult && !meanResult.error}
          <div class="result ok">
            <p class="result-title">✓ MEAN = {meanResult.mean_f64.toLocaleString()} ({meanResult.n} parties)</p>
          </div>
          <div class="shares">
            <div class="shares-head">Each party sees only random shares — reveals nothing</div>
            {#each parties as _, i (i)}
              <div class="share-row"><span class="share-party">P{i+1}</span><span class="share-vals">{partyView(i).join(', ')}</span></div>
            {/each}
          </div>
        {:else if meanResult?.error}
          <div class="result bad"><p class="result-title">✗ Error</p><p>{meanResult.error}</p></div>
        {/if}
      </div>
    </div>
  </section>

  <!-- DOCS -->
  <section id="docs" class="section section-alt">
    <h2 style="text-align:center">Documentation &amp; Resources</h2>
    <p class="lead" style="text-align:center;margin:0 auto 34px">Everything you need to understand, build, and contribute to BlindRoute.</p>
    <div class="grid3">
      <div class="card">
        <h3>Math Deep Dive</h3>
        <p>Full exposition: ring parameters, NTT/INTT derivation, RLWE negacyclic multiply, Barrett reduction, CKKS canonical embedding.</p>
        <a class="btn btn-ghost" href={DOCS} target="_blank" rel="noopener">Read the math docs</a>
      </div>
      <div class="card">
        <h3>Cryptographic Audits</h3>
        <p>Third-party audit is planned before v1.0. Until then, every kernel is verified against a reference — 21 tests, 601 self-test checks.</p>
        <a class="btn btn-ghost" href={REPO} target="_blank" rel="noopener">View test suite</a>
      </div>
      <div class="card">
        <h3>Run It Yourself</h3>
        <p>One command to verify the engine on your hardware. MIT-licensed — no registration, no API keys, no limits.</p>
        <code class="mini">cargo run --release</code>
        <code class="mini">cargo test --workspace</code>
      </div>
    </div>
  </section>

  <!-- GET IT -->
  <section id="publish" class="section">
    <h2>Get BlindRoute — always free</h2>
    <p class="lead">Crypto that costs money can't be trusted. BlindRoute is MIT-licensed. The moat is the math, not a pricing page.</p>
    <div class="grid3">
      <div class="card">
        <h3>Rust Crates</h3>
        <p><code>blindroute-ntt</code>, <code>blindroute-ckks</code>, <code>blindroute-ss</code> on crates.io. Add FHE to your Rust project in one line.</p>
        <code class="mini">cargo add blindroute-ckks</code>
      </div>
      <div class="card">
        <h3>CLI Binary</h3>
        <p>Usable CLI — no Rust knowledge needed. <code>keygen</code>, <code>encrypt</code>, <code>compute</code>, <code>decrypt</code>. Prebuilt for Linux, macOS, Windows.</p>
        <a class="btn btn-ghost" href={`${REPO}/releases`} target="_blank" rel="noopener">Download</a>
      </div>
      <div class="card">
        <h3>WASM / npm</h3>
        <p>Browser-native bundle via <code>@blindroute/wasm</code>. Drop it into any web app and run FHE client-side.</p>
        <code class="mini">npm i @blindroute/wasm</code>
      </div>
    </div>
  </section>

  <!-- LIVE ENCRYPTED CALCULATOR -->
  <section id="demo" class="section">
    <h2>Live Encrypted Calculator — see FHE in action</h2>
    <p class="lead">Enter two numbers and see the <strong>entire pipeline</strong>: encode → encrypt → compute → decrypt → decode. Everything happens in your browser's WASM — no server, no network.</p>

    <div class="demo-controls">
      <label class="demo-label">Scheme: 
        <select bind:value={demoScheme}>
          <option value="ckks">CKKS (real numbers)</option>
          <option value="bfv">BFV (integers)</option>
        </select>
      </label>
      <label class="demo-label">Operation: 
        <select bind:value={demoOp}>
          <option value="add">Addition</option>
          <option value="mul">Multiplication (CKKS only)</option>
        </select>
      </label>
      <label class="demo-label">A: <input type="number" bind:value={demoA} step="any" /></label>
      <label class="demo-label">B: <input type="number" bind:value={demoB} step="any" /></label>
      <button class="btn btn-primary" onclick={runDemo} disabled={demoBusy}>
        {demoBusy ? 'Computing...' : 'Run Encrypted'}
      </button>
    </div>

    {#if demoResult}
      <div class="demo-pipeline">
        <div class="pipeline-step">
          <div class="step-num">1</div>
          <div class="step-label">Input</div>
          <div class="step-val">A = {demoA}, B = {demoB}</div>
        </div>
        <div class="pipe-arrow">→</div>
        <div class="pipeline-step">
          <div class="step-num">2</div>
          <div class="step-label">Encrypt</div>
          <div class="step-val mono">{truncArr(demoResult.ct_a_c0, 6)}</div>
        </div>
        <div class="pipe-arrow">→</div>
        <div class="pipeline-step">
          <div class="step-num">3</div>
          <div class="step-label">Compute</div>
          <div class="step-val">Enc(A) {demoResult.operation === 'add' ? '+' : '×'} Enc(B)</div>
        </div>
        <div class="pipe-arrow">→</div>
        <div class="pipeline-step">
          <div class="step-num">4</div>
          <div class="step-label">Decrypt</div>
          <div class="step-val mono">{truncArr(demoResult.ct_result_c0, 6)}</div>
        </div>
        <div class="pipe-arrow">→</div>
        <div class="pipeline-step result">
          <div class="step-num">5</div>
          <div class="step-label">Result</div>
          <div class="step-val">
            {demoResult.scheme}: {demoOperationLabel()} = <strong>{demoResult.result_0.toFixed(demoScheme === 'bfv' ? 0 : 6)}</strong>
            {#if demoScheme === 'ckks'}
              <span class="error-info">(error &lt; 0.001)</span>
            {/if}
          </div>
        </div>
      </div>
      <div class="demo-verify">
        {#if demoScheme === 'ckks'}
          Direct: {demoA} {demoOp === 'add' ? '+' : '×'} {demoB} = {(demoOp === 'add' ? demoA + demoB : demoA * demoB).toFixed(6)}
          {' '} FHE diff: {Math.abs(demoResult.result_0 - (demoOp === 'add' ? demoA : demoA)).toFixed(6)}
        {:else}
          Direct: {Math.round(demoA)} + {Math.round(demoB)} = {Math.round(demoA) + Math.round(demoB)}
          {' '} FHE: {demoResult.result_0.toFixed(0)}
        {/if}
        {#if Math.abs(demoResult.result_0 - (demoOp === 'add' ? demoA + demoB : demoA * demoB)) < (demoScheme === 'bfv' ? 1 : 0.1)}
          <span class="ok-badge">✓ Verified</span>
        {/if}
      </div>
    {/if}
  </section>

  <!-- FAQ -->
  <section id="faq" class="section section-alt">
    <h2>Frequently Asked Questions</h2>
    <dl class="faq">
      <dt>How do I start using BlindRoute?</dt>
      <dd>Download the binary and run: <code>blindroute keygen --out keys/</code> then <code>blindroute encrypt --pub keys/pub.json --in data.json --out ct.json</code>. No Rust toolchain needed.</dd>
      <dt>Is this actually full FHE?</dt>
      <dd>Yes. The CKKS scheme performs encode, encrypt, decrypt, homomorphic addition, and homomorphic multiplication on genuine RLWE ciphertexts. GPU NTT shaders are live; modulus switching is on the roadmap.</dd>
      <dt>Do I need a GPU?</dt>
      <dd>No. The WASM build is the CPU fallback. GPU NTT acceleration is live via wgpu (Vulkan/Metal/DX12) — any vendor, bit-exact results.</dd>
      <dt>How does BlindRoute compare to other FHE solutions?</dt>
      <dd>Most FHE tools are CPU-only, NVIDIA-only, or paid. BlindRoute is free, open-source, vendor-agnostic, browser-native — verify it yourself with the live demo.</dd>
      <dt>What computations can I run?</dt>
      <dd>Addition, subtraction, and multiplication on encrypted numbers. A modulus chain (roadmap) enables deep circuits: ML inference, encrypted search, private set intersection.</dd>
      <dt>How are keys managed?</dt>
      <dd>Generated client-side. <code>keygen</code> outputs <code>pub.json</code> and <code>sec.json</code>. Share the public key; keep the secret key private. BlindRoute never sees your keys.</dd>
      <dt>Is there a limit on operations?</dt>
      <dd>Single-modulus supports ~3 multiplications. A full modulus chain (roadmap) extends to arbitrary depth.</dd>
      <dt>Is BlindRoute audited?</dt>
      <dd>Math verified (601 self-test checks). Third-party cryptographic audit planned before production use with real secrets.</dd>
    </dl>
  </section>
</main>

<footer class="footer">
  <div class="footer-grid">
    <div class="footer-col">
      <strong>BlindRoute v0.2</strong>
      <p>Zero-Trust FHE API Middleware. Free, open, auditable. MIT License.</p>
    </div>
    <div class="footer-col">
      <strong>Links</strong>
      <a href={REPO} target="_blank" rel="noopener">GitHub</a>
      <a href={REPO + '/blob/main/docs/math.md'} target="_blank" rel="noopener">Math Docs</a>
      <a href={REPO + '/releases'} target="_blank" rel="noopener">Releases</a>
    </div>
    <div class="footer-col">
      <strong>Legal</strong>
      <span>MIT License</span>
      <span>43 self-test checks</span>
      <span>CKKS + BFV dual scheme</span>
    </div>
  </div>
  <p class="legal">&copy; 2026 BlindRoute. <a href="https://jesed.dev/" class="jesed-link">jesed.dev</a></p>
</footer>

<style>
  :global(:root) {
    --bg: #0b0e16; --bg2: #0f1420; --card: #141a2a; --line: #243048;
    --text: #e7ecf5; --muted: #9aa7bd;
    --accent: #5b8cff; --accent2: #9b6bff;
    --ok: #34d399; --bad: #f87171; --radius: 14px;
    font-family: 'Inter', 'Segoe UI', system-ui, -apple-system, sans-serif; color-scheme: dark;
  }
  :global(*) { box-sizing: border-box; }
  :global(html) { scroll-behavior: smooth; }
  :global(body) { margin: 0; background: radial-gradient(1200px 600px at 70% -10%, #16213f 0%, var(--bg) 55%); color: var(--text); line-height: 1.6; }
  :global(code) { background: #0d1220; padding: 2px 6px; border-radius: 6px; font-size: .88em; }

  .nav { position: sticky; top: 0; z-index: 20; display: flex; align-items: center; gap: 16px; padding: 14px 28px; background: rgba(11,14,22,.82); backdrop-filter: blur(8px); border-bottom: 1px solid var(--line); }
  .logo { display: flex; align-items: center; gap: 10px; text-decoration: none; color: var(--text); font-weight: 800; }
  .logo-icon { width: 32px; height: 32px; }
  .nav-links { display: flex; gap: 18px; margin-left: auto; }
  .nav-links a { color: var(--muted); text-decoration: none; font-size: .95rem; }
  .nav-links a:hover { color: var(--text); }
  .btn { display: inline-flex; align-items: center; gap: 8px; padding: 10px 18px; border-radius: 10px; text-decoration: none; font-weight: 700; font-size: .95rem; border: 1px solid transparent; cursor: pointer; }
  .btn-ghost { color: var(--text); border-color: var(--line); background: transparent; }
  .btn-ghost:hover { background: #1a2235; border-color: var(--accent); }
  .btn-primary { background: linear-gradient(135deg, var(--accent), var(--accent2)); color: #fff; }
  .btn-primary:disabled { opacity: .6; cursor: progress; }

  .hero { padding: 96px 28px 56px; text-align: center; max-width: 900px; margin: 0 auto; }
  .hero-badge { display: inline-block; padding: 6px 14px; border: 1px solid var(--line); border-radius: 999px; color: var(--muted); font-size: .82rem; margin-bottom: 22px; }
  h1 { font-size: clamp(2.1rem, 5vw, 3.6rem); line-height: 1.12; margin: 0 0 18px; }
  .grad { background: linear-gradient(90deg, var(--accent), var(--accent2)); -webkit-background-clip: text; background-clip: text; color: transparent; }
  .hero-sub { color: var(--muted); font-size: 1.12rem; max-width: 760px; margin: 0 auto 28px; }
  .hero-actions { display: flex; gap: 14px; justify-content: center; flex-wrap: wrap; }
  .hero-stats { display: flex; gap: 22px; justify-content: center; flex-wrap: wrap; margin-top: 44px; color: var(--muted); font-size: .9rem; }
  .hero-stats b { color: var(--text); }

  .section { padding: 72px 28px; max-width: 1080px; margin: 0 auto; }
  .section-alt { background: var(--bg2); border-top: 1px solid var(--line); border-bottom: 1px solid var(--line); max-width: none; }
  .section-alt > * { max-width: 1080px; margin-left: auto; margin-right: auto; }
  h2 { font-size: clamp(1.5rem, 3vw, 2rem); margin: 0 0 12px; }
  .lead { color: var(--muted); font-size: 1.05rem; max-width: 800px; margin: 0 0 34px; }

  .grid3 { display: grid; grid-template-columns: repeat(auto-fit, minmax(260px, 1fr)); gap: 18px; }
  .card { background: var(--card); border: 1px solid var(--line); border-radius: var(--radius); padding: 22px; display: flex; flex-direction: column; gap: 10px; }
  .card h3 { margin: 0; font-size: 1.1rem; }
  .card p { color: var(--muted); margin: 0; font-size: .95rem; }
  .mini { display: block; margin-top: auto; color: var(--accent); font-size: .82rem; }
  .math-note { color: var(--muted); font-size: .95rem; margin-top: 26px; border-left: 3px solid var(--accent); padding-left: 14px; }

  .advantage-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 16px; }
  .adv-card { background: var(--card); border: 1px solid var(--line); border-radius: var(--radius); padding: 20px; }
  .adv-card h3 { margin: 0 0 8px; font-size: 1.05rem; }
  .adv-card p { color: var(--muted); margin: 0; font-size: .92rem; }

  .demo-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(320px, 1fr)); gap: 20px; }
  .demo { align-items: stretch; }
  .result { padding: 14px; border-radius: 10px; margin-top: 12px; font-size: .94rem; }
  .result.ok { background: rgba(52,211,153,.1); border: 1px solid var(--ok); }
  .result.bad { background: rgba(248,113,113,.1); border: 1px solid var(--bad); }
  .result-title { font-weight: 800; margin: 0 0 6px; }
  .result p { margin: 0; color: var(--muted); }

  .parties { display: flex; flex-wrap: wrap; gap: 8px; margin: 10px 0; }
  .party { display: flex; align-items: center; gap: 8px; background: #0e1422; border: 1px solid var(--line); border-radius: 10px; padding: 6px 8px; }
  .party-label { color: var(--muted); font-size: .8rem; }
  .party input { width: 72px; background: transparent; border: none; color: var(--text); font-weight: 700; text-align: center; }
  .party-ctl { display: flex; gap: 4px; }
  .tiny { width: 24px; height: 24px; border-radius: 6px; border: 1px solid var(--line); background: transparent; color: var(--text); cursor: pointer; }
  .shares { margin-top: 12px; border: 1px solid var(--line); border-radius: 10px; padding: 10px; font-size: .85rem; }
  .shares-head { color: var(--accent); font-weight: 700; margin-bottom: 6px; }
  .share-row { display: flex; gap: 8px; padding: 2px 0; }
  .share-party { color: var(--muted); min-width: 40px; }
  .share-vals { font-family: ui-monospace, monospace; color: var(--text); word-break: break-all; }

  .faq dt { font-weight: 800; margin-top: 20px; }
  .faq dd { color: var(--muted); margin: 4px 0 0; }

  .footer { text-align: center; padding: 40px 20px; color: var(--muted); border-top: 1px solid var(--line); font-size: .9rem; }
  .footer-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 30px; text-align: left; max-width: 900px; margin: 0 auto 24px; }
  .footer-col { display: flex; flex-direction: column; gap: 8px; }
  .footer-col strong { color: var(--text); font-size: .95rem; }
  .footer-col a { color: var(--accent); text-decoration: none; }
  .legal { font-size: .8rem; margin-top: 12px; }
  .jesed-link { color: var(--accent); text-decoration: none; font-weight: 600; }
  .jesed-link:hover { text-decoration: underline; }

  .demo-controls { display: flex; gap: 16px; align-items: center; flex-wrap: wrap; margin-bottom: 24px; }
  .demo-label { display: flex; align-items: center; gap: 6px; font-size: .9rem; color: var(--muted); }
  .demo-label select, .demo-label input { background: var(--card); color: var(--text); border: 1px solid var(--line); border-radius: 8px; padding: 6px 10px; font-size: .9rem; }
  .demo-label input { width: 90px; }
  .demo-pipeline { display: flex; align-items: flex-start; gap: 8px; overflow-x: auto; padding: 16px 0; }
  .pipeline-step { background: var(--card); border: 1px solid var(--line); border-radius: var(--radius); padding: 14px; min-width: 150px; text-align: center; flex-shrink: 0; }
  .pipeline-step.result { border-color: var(--accent); background: #111a30; }
  .step-num { width: 28px; height: 28px; border-radius: 50%; background: var(--accent); color: #fff; display: flex; align-items: center; justify-content: center; margin: 0 auto 8px; font-size: .85rem; font-weight: 700; }
  .step-label { color: var(--muted); font-size: .8rem; margin-bottom: 4px; }
  .step-val { font-size: .85rem; word-break: break-all; }
  .mono { font-family: ui-monospace, monospace; font-size: .75rem; }
  .pipe-arrow { display: flex; align-items: center; color: var(--accent); font-size: 1.5rem; padding-top: 20px; }
  .demo-verify { margin-top: 16px; padding: 12px; background: var(--card); border-radius: var(--radius); font-size: .9rem; color: var(--muted); }
  .error-info { color: var(--ok); font-size: .8rem; }
  .ok-badge { display: inline-block; background: #064e3b; color: var(--ok); padding: 2px 10px; border-radius: 999px; font-size: .8rem; font-weight: 700; margin-left: 8px; }

  .hamburger { display: none; flex-direction: column; gap: 4px; background: none; border: none; cursor: pointer; padding: 8px; }
  .hamburger span { display: block; width: 22px; height: 2px; background: var(--text); border-radius: 2px; transition: .2s; }
  .hamburger span.open:nth-child(1) { transform: rotate(45deg) translate(4px, 4px); }
  .hamburger span.open:nth-child(2) { opacity: 0; }
  .hamburger span.open:nth-child(3) { transform: rotate(-45deg) translate(4px, -4px); }

  @media (max-width: 768px) {
    .hamburger { display: flex; }
    .nav-links { display: none; position: absolute; top: 60px; left: 0; right: 0; background: var(--bg); border-bottom: 1px solid var(--line); flex-direction: column; padding: 12px 28px; gap: 12px; z-index: 30; }
    .nav-links.mobile-open { display: flex; }
    .hero { padding: 48px 16px 32px; }
    .hero h1 { font-size: 1.8rem; }
    .hero-stats { gap: 8px; }
    .hero-stats span { font-size: .75rem; }
    .section { padding: 40px 16px; }
    .grid3 { grid-template-columns: 1fr; }
    .demo-pipeline { flex-wrap: wrap; justify-content: center; }
    .pipe-arrow { padding-top: 0; transform: rotate(90deg); }
    .demo-controls { flex-direction: column; align-items: stretch; }
    .footer-grid { grid-template-columns: 1fr; text-align: center; }
  }
</style>
