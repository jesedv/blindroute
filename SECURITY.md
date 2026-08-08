# Security Policy

## Status

**RingCrypt is pre-audit (v0.1).** The mathematics are verified against reference
implementations and self-tests, but the engine has **not** undergone a formal
third-party cryptographic audit.

**Do not use RingCrypt to protect real secrets until v1.0 has passed an audit.**

## Reporting a vulnerability

This project touches cryptography. Please report security issues privately:

- Open a [private advisory](https://github.com/jesedv/ringcrypt/security/advisories/new)
- or email the maintainers (see repo About)

We aim to acknowledge reports within 48 hours and to respond publicly once a
fix/privacy window is appropriate.

## Hard rules

- `#![forbid(unsafe_code)]` in every core crate — no unsafe in the critical path.
- Every kernel ships a reference cross-check (`cargo test`, `scripts/regress-ntt.sh`,
  and the in-browser `run_self_test`).
- No LLM-generated cryptographic or numerical code is accepted into `crates/`.
