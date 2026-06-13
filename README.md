# Ternary Codes

**Ternary Codes** implements error-correcting codes for ternary data in {-1, 0, +1} — providing Hamming codes over GF(3), repetition codes, parity checks, syndrome decoding, and code distance calculation.

## Why It Matters

Noise corrupts data in storage, transmission, and computation. Error-correcting codes detect and correct this noise. While binary codes (Hamming, Reed-Solomon) are well-established, ternary codes over GF(3) offer advantages: more efficient packing (log₂3 ≈ 1.585 bits per trit vs 1 bit), better error-correcting capability for burst errors, and natural compatibility with ternary arithmetic hardware. The ternary Golay code is one of the most beautiful objects in coding theory — a perfect code with deep connections to combinatorics.

## How It Works

### GF(3) Arithmetic

All operations in the Galois field GF(3) = {0, 1, 2}:

```
Addition mod 3:
  + | 0  1  2
  0 | 0  1  2
  1 | 1  2  0
  2 | 2  0  1

Multiplication mod 3:
  × | 0  1  2
  0 | 0  0  0
  1 | 0  1  2
  2 | 0  2  1
```

Mapping to ternary: 0→Zero, 1→Pos, 2→Neg. All GF(3) operations: **O(1)**.

### Ternary Hamming Code

A [n, k, d] code over GF(3) with:
- n = (3^r - 1) / 2 code symbols (for r parity symbols)
- k = n - r information symbols
- d = 3 minimum distance (corrects 1 error)

**Encoding:**
```
codeword = data · G    (G = generator matrix, n×k over GF(3))
```

Matrix multiplication: **O(n·k)**.

**Syndrome Decoding:**
```
syndrome = received · H^T   (H = parity check matrix, r×n)
if syndrome == 0: no errors
else: look up error position in syndrome table
correct the error
```

Syndrome computation: **O(r·n)**. Error lookup: **O(1)** (precomputed table). Single-error correction: **O(r·n)** total.

### Repetition Code

Each trit repeated N times:
```
encode(x) = [x, x, x, ..., x]  (N copies)
decode = majority vote
```

Encoding: **O(N)**. Decoding: **O(N)**. Corrects ⌊(N-1)/2⌋ errors.

### Parity Check

Append one check trit so the sum of all trits is 0 mod 3:
```
parity = -(Σ data) mod 3
codeword = data ∥ [parity]
```

Detects 2 errors, corrects 0. Encoding/decoding: **O(N)**.

### Code Distance

Minimum Hamming distance between any two codewords:
```
d_min = min { d_H(c_i, c_j) : i ≠ j }
```

Error-correcting capacity: t = ⌊(d-1)/2⌋.
Error-detecting capacity: d-1.

Brute force distance: **O(M² · N)** for M codewords of length N.

## Quick Start

```rust
use ternary_codes::{Ternary, HammingCode, hamming_encode, hamming_decode};

let data = vec![Ternary::Pos, Ternary::Zero, Ternary::Neg, Ternary::Pos];
let encoded = hamming_encode(&data);  // Adds parity trits
let mut corrupted = encoded.clone();
corrupted[2] = Ternary::Zero;  // Introduce error
let decoded = hamming_decode(&corrupted).unwrap();  // Corrects error
assert_eq!(decoded, data);
```

## API

| Function | Complexity | Description |
|----------|------------|-------------|
| `ternary_add_mod3(a, b)` | O(1) | GF(3) addition |
| `ternary_mul_mod3(a, b)` | O(1) | GF(3) multiplication |
| `hamming_encode(data)` | O(n·k) | Encode with parity symbols |
| `hamming_decode(received)` | O(r·n) | Syndrome decode and correct |
| `repetition_encode(x, n)` | O(N) | Repeat N times |
| `repetition_decode(codeword)` | O(N) | Majority vote decode |
| `parity_check(data)` | O(N) | Append parity trit |
| `min_distance(codewords)` | O(M²·N) | Minimum code distance |

## Architecture Notes

Ternary Codes provides the error-correction layer for reliable ternary data transmission in SuperInstance. In γ + η = C, error correction preserves the conservation invariant C despite noise: the code ensures that γ and η values are correctly transmitted and stored. Integrates with `ternary-cipher` for authenticated error-correcting channels and `superinstance-protocol` for bottle payload integrity.

See [ARCHITECTURE.md](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md) for reliable communication architecture.

## References

1. Hamming, R. W. (1950). "Error Detecting and Error Correcting Codes." *Bell System Technical Journal*, 29(2), 147–160.
2. MacWilliams, F. J. & Sloane, N. J. A. (1977). *The Theory of Error-Correcting Codes*. North-Holland.
3. Conway, J. H. & Sloane, N. J. A. (1993). *Sphere Packings, Lattices and Groups*, 2nd ed. Springer.

## License

MIT
