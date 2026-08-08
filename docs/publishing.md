# Publishing & SEO plan

This document is the release playbook for RingCrypt. It answers the question
"should we publish the binary or just the project?" with: **both — plus the
site, packages, and a complete SEO surface.**

## 1. Release a binary AND the project

The moat is the math, not licensing, so open everything:

| Artifact | Why | Trigger |
|---|---|---|
| **Source repo (MIT)** | Trust model. Crypto is only trusted when auditable. | GitHub push |
| **Native binaries** (Linux/macOS/Windows) | Non-Rust users verify without a toolchain. | push tag `v*` (`.github/workflows/release.yml`) |
| **WASM bundle in Releases** | Self-host the demo, no Rust needed. | same tag |
| **crates.io** `ringcrypt-ntt`, `ringcrypt-ss` | Developer adoption. | after audit/first stable |
| **npm** `@ringcrypt/wasm` | Front-end adoption. | after demo stabilizes |
| **Static site** (Svelte → GitHub Pages) | Public proof + SEO. | push to `main` (`pages.yml`) |

Revenue is separate: third-party audits, a managed RingCrypt cloud, and
enterprise support. Open-sourcing the code does not threaten that.

## 2. How to do the GitHub release

1. `git tag v0.1.0 && git push origin v0.1.0`.
2. `release.yml` cross-compiles the CLI for 4 targets and stages the WASM
   bundle, then opens a **draft release** with release notes.
3. Review the draft, add a human summary + benchmarks, publish.
4. Repeat at `v1.0.0` **only after** the third-party security audit.

## 3. SEO — on the page

The landing page (`web/index.html`, `src/App.svelte`) ships:

- Unique `<title>` + meta `description`/`keywords` targeting *homomorphic
  encryption*, *FHE*, *NTT*, *privacy*, *GPU*, *WASM*, *RLWE/CKKS/BFV*.
- **Open Graph + Twitter card** (`og-cover.svg`) for link previews on socials.
- **JSON-LD `SoftwareSourceCode`** structured data for rich results.
- Canonical URL, `robots.txt`, and `sitemap.xml`.
- `<noscript>` fallback with full descriptive content (crawlers/clients with JS
  off still see what the project is).

## 4. SEO — on the repository

- **Name & description**: repo named `ringcrypt`; the GitHub description / About
  should read: *"Cross-vendor fully homomorphic encryption on GPU — free,
  open-source, browser-native NTT + secret-sharing engine (RLWE/CKKS/BFV-style)."*
- **Topics**: `fhe`, `homomorphic-encryption`, `ntt`, `fully-homomorphic-encryption`,
  `cryptography`, `rust`, `wasm`, `privacy`, `rlwe`, `ckks`, `gpu`, `webgpu`, `wasm-bindgen`.
- **README**: badge header, "why", "how it works", quick start, live demo link,
  license — so the repo answers *itself* in searches and first-reads.
- **Website field** on the repo → the Pages URL; **Releases**, **Pages**, **Actions**
  tabs all enabled and populated.
- Metadata lives in `Cargo.toml` (`repository`, `homepage`, `description`,
  `readme`) so crates.io/auto-generated pages carry it too.

## 5. Tips before going public

- Enable GitHub Pages → branch `gh-pages` (or Actions deploy; `pages.yml` uses
  the Actions path).
- Add the License, contributing guide, and a security policy (`SECURITY.md`)
  before announcing.
- Announce on a broad audience (privacy/crypto, Rust, WASM/WebGPU) with a link
  to the live demo — the working browser demo is the strongest signal.
