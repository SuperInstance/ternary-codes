# Future Integration: ternary-codes

## Current State
Provides error-correcting codes for ternary data: Hamming codes over GF(3), repetition codes, parity checks, and code distance calculation. All operations work on ternary values mapped to GF(3) ({Neg→2, Zero→0, Pos→1}) with mod-3 arithmetic.

## Integration Opportunities

### With ternary-protocol (Message Integrity)
ternary-protocol's `TernaryMessage` is a trit sequence transmitted over unreliable channels (UART to ESP32, network to Codespace). ternary-codes' `TernaryHamming` can encode message payloads with error correction before transmission. The code distance calculation tells you how many bit-flips (trit-flips) a message can survive — critical for noisy ESP32 UART links where electromagnetic interference corrupts trits.

### With ternary-steganography (Covert Channels)
The cross-pollination report notes that ternary-protocol messages are trit sequences and ternary-steganography embeds data in trit sequences. Error-correcting codes enable this: by encoding messages with redundancy, the redundant trits become the steganographic carrier. A `TernaryHamming` codeword carries 2 data trits + 3 parity trits — the parity trits can be deliberately chosen to embed hidden data without breaking error correction.

### With ternary-adversarial (Adversarial Noise)
ternary-adversarial perturbs trits to flip decisions. ternary-codes' minimum distance computation measures exactly how many perturbations a code can detect/correct. A strategy whose decision function is encoded as a ternary codeword has quantified adversarial robustness: the code distance IS the minimum perturbation budget an adversary needs.

## Potential in Mature Systems
In room-as-codespace, every `TernaryMessage` between rooms (Codespaces) uses error-correcting encoding. When PLATO synchronizes tiles between rooms over HTTPS, the payload includes ternary parity checks. When the ESP32 sends sensor readings over UART, `TernaryHamming` protects against corruption. The code rate (data trits / total trits) becomes a tunable parameter: high reliability rooms use aggressive coding (rate 2/5), low-latency rooms use minimal coding (rate 4/5).

## Cross-Pollination Ideas
- **ternary-kalman**: Kalman filter state estimates are uncertain. Encode the estimate as a ternary codeword where uncertainty maps to the code's error-correction capacity.
- **ternary-federated**: Federated aggregation of ternary strategies over noisy channels needs error correction to prevent Byzantine-style corruption from being indistinguishable from channel noise.
- **ternary-thermodynamics**: Entropy of the code's symbol distribution relates to channel capacity — Shannon's theorem in ternary.

## Dependencies for Next Steps
- Add GF(3) encoding to ternary-protocol's `TernaryMessage` serialization
- Benchmark Hamming encoding/decoding on ESP32 (must be <1ms per message)
- Integrate code distance into ternary-adversarial's `RobustnessReport`
- Define `CodedPayload` wrapper type in ternary-protocol
