# ternary-codes

Error-correcting codes for ternary data — Hamming codes, repetition codes, parity checks, and distance metrics for the domain {-1, 0, +1}.

## Why This Exists

Classical coding theory operates over binary (GF(2)) or larger prime fields. Ternary codes — working over GF(3) — are less common but arise in quantum computing (qutrits), ternary logic circuits, balanced ternary arithmetic, and multi-level flash storage. This crate brings proven coding-theory primitives to the ternary setting: Hamming codes with ternary parity, repetition codes with majority-vote decoding, parity encoding/verification, and both Hamming and Lee distance metrics. It's `no_std` and `forbid(unsafe_code)`.

## Core Concepts

- **Ternary arithmetic mod 3**: Addition and subtraction in GF(3), mapping `Neg→2`, `Zero→0`, `Pos→1`.
- **TernaryHamming**: Systematic Hamming code with `r` check symbols, block length `n = (3ʳ − 1)/2`. Corrects single-symbol errors.
- **RepetitionCode**: Repeats each symbol `n` times (n ≥ 3), corrects up to ⌊(n−1)/2⌋ errors per symbol via majority vote.
- **Parity encoding**: Single parity symbol via sum mod 3; detects single-symbol errors.
- **Distance metrics**: Hamming distance (positions that differ), Lee distance (circular distance in GF(3)), codeword weight (non-zero symbols).
- **Minimum distance**: Compute the minimum distance of a codebook and derive error correction/detection capacity.

## Quick Start

```toml
# Cargo.toml
[dependencies]
ternary-codes = "0.1"
```

```rust
use ternary_codes::{
    Ternary, RepetitionCode, parity_encode, parity_verify,
    hamming_distance, minimum_distance, error_correction_capacity,
};

fn main() {
    // Parity encoding
    let data = vec![Ternary::Pos, Ternary::Pos, Ternary::Neg];
    let encoded = parity_encode(&data);
    assert!(parity_verify(&encoded));

    // Repetition code: 5-fold repetition, corrects up to 2 errors per symbol
    let code = RepetitionCode::new(5);
    let encoded = code.encode(&data);
    let decoded = code.decode(&encoded).unwrap();
    assert_eq!(decoded, data);

    // Distance metrics
    let a = vec![Ternary::Pos, Ternary::Zero, Ternary::Neg];
    let b = vec![Ternary::Neg, Ternary::Zero, Ternary::Pos];
    assert_eq!(hamming_distance(&a, &b), Some(2));

    let codebook = vec![
        vec![Ternary::Pos, Ternary::Pos, Ternary::Pos],
        vec![Ternary::Neg, Ternary::Neg, Ternary::Neg],
        vec![Ternary::Zero, Ternary::Zero, Ternary::Zero],
    ];
    let d_min = minimum_distance(&codebook); // Some(3)
    assert_eq!(error_correction_capacity(d_min.unwrap()), 1);
}
```

## API Overview

| Type / Function | Description |
|---|---|
| `Ternary` | Value enum: `Neg`, `Zero`, `Pos`, with `add_mod3`, `sub_mod3` |
| `TernaryHamming` | Encode/decode with `r` check symbols, single-error correction |
| `RepetitionCode` | N-fold repetition with majority-vote decoding |
| `parity_encode` / `parity_verify` | Single parity symbol encoding and verification |
| `ternary_parity` / `parity_check` | Low-level parity computation |
| `hamming_distance` | Position-count distance between codewords |
| `lee_distance` / `lee_distance_codeword` | Circular (GF(3)) distance |
| `minimum_distance` | Min pairwise distance in a codebook |
| `error_correction_capacity` / `error_detection_capacity` | Derived from minimum distance |
| `weight` | Number of non-zero symbols |
| `majority_vote` | Resolve a slice of ternary values to the most common |

## How It Works

All operations work over GF(3), with `Neg → 2`, `Zero → 0`, `Pos → 1`. The **TernaryHamming** code constructs a parity-check matrix from non-zero vectors in GF(3)ʳ (one representative per scalar-multiple equivalence class). Encoding is systematic (data + parity). Decoding computes the syndrome and brute-force tests single-symbol corrections.

The **RepetitionCode** simply repeats each symbol `n` times and decodes via `majority_vote`, counting occurrences of each value. **Lee distance** treats the ternary alphabet as circular: `d(Neg, Pos) = 1` (not 2), which better reflects GF(3) geometry.

## Use Cases

- **Quantum qutrit error correction**: Ternary codes are natural for qutrit-based quantum computing.
- **Multi-level flash storage**: 3-level cell (TLC) storage benefits from ternary coding-theory approaches.
- **Balanced ternary communication**: Error-correcting codes for balanced ternary signaling schemes.
- **Ternary logic circuit testing**: Parity and repetition codes for built-in self-test (BIST) of ternary circuits.

## Known Limitations

- **TernaryHamming decoding is brute-force**: The `decode()` method computes the syndrome and then iterates over all possible single-symbol error positions and values to find the correction. This is O(n × 3) per codeword, which is fine for the short block lengths ternary Hamming codes produce (n ≤ 13 for r=3), but would not scale to longer codes.

- **Block lengths are limited by the Hamming formula**: `n = (3ʳ − 1)/2` grows exponentially with `r`, but `r` is stored as `usize`. For r ≥ 20, `3ʳ` overflows `usize` on 64-bit platforms. The constructor does not check for overflow, so `TernaryHamming::new(40)` would silently compute a wrong block length.

- **RepetitionCode has no streaming interface**: `encode()` and `decode()` require the full input upfront. For long data streams, you must buffer the entire message before decoding, which defeats the purpose of streaming error correction.

- **Majority vote can return Zero on true ties**: When a symbol is repeated an even number of times and votes are split (e.g., 2 Pos, 2 Neg, 1 Zero), `majority_vote()` returns `Zero`. This may not be the desired behavior — some applications prefer to flag the ambiguity rather than silently resolve to the neutral value.

- **Parity encoding only detects single-symbol errors**: `parity_verify()` computes the sum mod 3 and compares against the check symbol. Two-symbol errors (which are the most common in burst-error channels) can cancel out in GF(3), making them undetectable.

- **`minimum_distance()` is O(m² × n)**: Computing minimum pairwise distance across a codebook with `m` codewords of length `n` requires comparing all pairs. For large codebooks (m > 1000), this is slow. No caching or early-exit optimization is implemented.

## Ecosystem

Part of the **SuperInstance** ternary computing suite:

- `ternary-lattice` — lattice structures for ternary values
- `ternary-codes` — this crate
- `ternary-gradient` — gradient-free optimization on ternary landscapes
- `ternary-language` — ternary NLP and grammar processing
- `ternary-trees` — ternary decision trees and forests
- `ternary-transform` — wavelet, Fourier, and kernel transforms
- `ternary-planning` — planning and scheduling with ternary priorities
- `ternary-rl` — reinforcement learning with ternary actions
- `ternary-som` — self-organizing maps for ternary data
- `ternary-failure` — failure analysis with ternary classification

## License

MIT

## See Also
- **ternary-hash** — related
- **ternary-steganography** — related
- **ternary-entropy** — related
- **ternary-compression** — related
- **ternary-ring** — related

