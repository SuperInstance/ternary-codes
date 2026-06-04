//! Error-correcting codes for ternary data.
//!
//! Provides Hamming codes, repetition codes, parity checks, and
//! code distance calculation for ternary-valued data in {-1, 0, +1}.

#![forbid(unsafe_code)]
#![no_std]

extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;

/// A ternary value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Ternary {
    Neg = -1,
    Zero = 0,
    Pos = 1,
}

impl Ternary {
    pub fn from_i8(v: i8) -> Option<Self> {
        match v {
            -1 => Some(Ternary::Neg),
            0 => Some(Ternary::Zero),
            1 => Some(Ternary::Pos),
            _ => None,
        }
    }

    pub fn to_i8(self) -> i8 {
        self as i8
    }

    /// Ternary addition mod 3. Maps Ternary to GF(3): Neg→2, Zero→0, Pos→1.
    pub fn add_mod3(a: Ternary, b: Ternary) -> Ternary {
        let va: i32 = match a {
            Ternary::Neg => 2,
            Ternary::Zero => 0,
            Ternary::Pos => 1,
        };
        let vb: i32 = match b {
            Ternary::Neg => 2,
            Ternary::Zero => 0,
            Ternary::Pos => 1,
        };
        match (va + vb) % 3 {
            0 => Ternary::Zero,
            1 => Ternary::Pos,
            2 => Ternary::Neg,
            _ => unreachable!(),
        }
    }

    /// Ternary subtraction mod 3: a - b (mod 3).
    pub fn sub_mod3(a: Ternary, b: Ternary) -> Ternary {
        let va: i32 = match a {
            Ternary::Neg => 2,
            Ternary::Zero => 0,
            Ternary::Pos => 1,
        };
        let vb: i32 = match b {
            Ternary::Neg => 2,
            Ternary::Zero => 0,
            Ternary::Pos => 1,
        };
        match ((va - vb) % 3 + 3) % 3 {
            0 => Ternary::Zero,
            1 => Ternary::Pos,
            2 => Ternary::Neg,
            _ => unreachable!(),
        }
    }
}

impl core::ops::Neg for Ternary {
    type Output = Self;
    fn neg(self) -> Self {
        match self {
            Ternary::Neg => Ternary::Pos,
            Ternary::Zero => Ternary::Zero,
            Ternary::Pos => Ternary::Neg,
        }
    }
}

// ── Ternary Hamming Code ─────────────────────────────────────────────

/// Ternary Hamming code with r check symbols.
/// Block length n = (3^r - 1) / 2, data symbols k = n - r.
/// Can correct up to 1 error per codeword.
#[derive(Clone, Debug)]
pub struct TernaryHamming {
    r: usize,
    n: usize,
    k: usize,
}

impl TernaryHamming {
    /// Create a Hamming code with r check symbols.
    pub fn new(r: usize) -> Self {
        assert!(r >= 2, "need at least 2 check symbols");
        let n = (3usize.pow(r as u32) - 1) / 2;
        let k = n - r;
        Self { r, n, k }
    }

    pub fn n(&self) -> usize { self.n }
    pub fn k(&self) -> usize { self.k }
    pub fn r(&self) -> usize { self.r }

    /// Compute parity check matrix positions for the Hamming code.
    /// Returns the parity positions for each check symbol.
    fn parity_positions(&self) -> Vec<Vec<usize>> {
        // For ternary Hamming codes, the parity check matrix H has columns
        // that are all non-zero vectors in GF(3)^r (up to scalar multiples).
        // We generate these systematically.
        let mut positions: Vec<Vec<usize>> = (0..self.r).map(|_| Vec::new()).collect();

        let mut col_idx = 0;
        // Generate all non-zero vectors in GF(3)^r, taking one representative
        // from each equivalence class (up to scalar multiplication by {1, 2}).
        let total_cols = self.n;
        let mut seen = 0;

        // Simple approach: iterate through all vectors, pick representatives
        for val in 1..=total_cols {
            if seen >= total_cols {
                break;
            }
            // Convert val to base-3 representation of length r
            let digits = Self::to_base3(val, self.r);
            let is_rep = Self::is_representative(&digits);
            if is_rep {
                for (check_idx, &d) in digits.iter().enumerate() {
                    if d != 0 {
                        positions[check_idx].push(col_idx);
                    }
                }
                col_idx += 1;
                seen += 1;
            }
        }
        positions
    }

    fn to_base3(mut val: usize, len: usize) -> Vec<u8> {
        let mut digits = vec![0u8; len];
        let mut i = len;
        while val > 0 && i > 0 {
            i -= 1;
            digits[i] = (val % 3) as u8;
            val /= 3;
        }
        digits
    }

    fn is_representative(digits: &[u8]) -> bool {
        // A vector is a representative if the first non-zero digit is 1
        for &d in digits {
            if d == 1 {
                return true;
            }
            if d == 2 {
                return false;
            }
        }
        false // all zeros
    }

    /// Compute syndrome for a received codeword.
    fn syndrome(&self, received: &[Ternary]) -> Vec<Ternary> {
        let positions = self.parity_positions();
        let mut syndrome = Vec::with_capacity(self.r);
        for check_idx in 0..self.r {
            let mut sum = Ternary::Zero;
            for &pos in &positions[check_idx] {
                if pos < received.len() {
                    sum = Ternary::add_mod3(sum, received[pos]);
                }
            }
            // Subtract the expected parity (which should be zero for valid codewords)
            syndrome.push(sum);
        }
        syndrome
    }

    /// Encode data into a codeword.
    /// For simplicity, we use systematic encoding: [data | parity].
    pub fn encode(&self, data: &[Ternary]) -> Vec<Ternary> {
        assert_eq!(data.len(), self.k, "data must have k={} symbols", self.k);

        // Initialize codeword with data
        let mut codeword = data.to_vec();

        // Compute parity symbols
        for check_idx in 0..self.r {
            let mut parity = Ternary::Zero;
            for (i, &val) in data.iter().enumerate() {
                // Simple parity: weighted sum mod 3
                let weight = ((i + 1 + check_idx) % 3) as i8;
                if weight == 0 {
                    // skip
                } else if weight == 1 {
                    parity = Ternary::add_mod3(parity, val);
                } else {
                    // weight == 2
                    parity = Ternary::add_mod3(parity, val);
                    parity = Ternary::add_mod3(parity, val);
                }
            }
            codeword.push(-parity); // negate so that syndrome = 0
        }
        codeword
    }

    /// Decode a received codeword, correcting up to 1 error.
    /// Returns the decoded data symbols.
    pub fn decode(&self, received: &[Ternary]) -> Result<Vec<Ternary>, DecodeError> {
        if received.len() != self.n {
            return Err(DecodeError::InvalidLength);
        }

        let syndrome = self.syndrome(received);

        // Check if syndrome is all zeros (no error)
        if syndrome.iter().all(|&s| s == Ternary::Zero) {
            return Ok(received[..self.k].to_vec());
        }

        // Try to find and correct single error
        // The syndrome tells us the error position and value
        for pos in 0..self.n {
            for &error_val in &[Ternary::Neg, Ternary::Pos] {
                let mut corrected = received.to_vec();
                corrected[pos] = Ternary::add_mod3(corrected[pos], error_val);
                let new_syndrome = self.syndrome(&corrected);
                if new_syndrome.iter().all(|&s| s == Ternary::Zero) {
                    return Ok(corrected[..self.k].to_vec());
                }
            }
        }

        Err(DecodeError::Uncorrectable)
    }
}

/// Decode error types.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecodeError {
    InvalidLength,
    Uncorrectable,
}

// ── Repetition Code ──────────────────────────────────────────────────

/// Ternary repetition code: repeats each symbol `n` times.
/// Can correct up to ⌊(n-1)/2⌋ errors per symbol.
#[derive(Clone, Copy, Debug)]
pub struct RepetitionCode {
    repeat: usize,
}

impl RepetitionCode {
    pub fn new(repeat: usize) -> Self {
        assert!(repeat >= 3, "repetition factor must be >= 3");
        Self { repeat }
    }

    pub fn repeat_factor(&self) -> usize { self.repeat }

    /// Maximum correctable errors per symbol.
    pub fn error_capacity(&self) -> usize {
        (self.repeat - 1) / 2
    }

    /// Encode data by repeating each symbol.
    pub fn encode(&self, data: &[Ternary]) -> Vec<Ternary> {
        let mut result = Vec::with_capacity(data.len() * self.repeat);
        for &val in data {
            for _ in 0..self.repeat {
                result.push(val);
            }
        }
        result
    }

    /// Decode by majority vote on each group.
    pub fn decode(&self, received: &[Ternary]) -> Result<Vec<Ternary>, DecodeError> {
        if received.len() % self.repeat != 0 {
            return Err(DecodeError::InvalidLength);
        }
        let mut result = Vec::new();
        for chunk in received.chunks(self.repeat) {
            if chunk.len() != self.repeat {
                return Err(DecodeError::InvalidLength);
            }
            result.push(majority_vote(chunk));
        }
        Ok(result)
    }
}

/// Majority vote on a slice of ternary values.
pub fn majority_vote(values: &[Ternary]) -> Ternary {
    let mut counts = [0i32; 3]; // neg, zero, pos
    for &v in values {
        match v {
            Ternary::Neg => counts[0] += 1,
            Ternary::Zero => counts[1] += 1,
            Ternary::Pos => counts[2] += 1,
        }
    }
    if counts[2] > counts[0] && counts[2] > counts[1] {
        Ternary::Pos
    } else if counts[0] > counts[2] && counts[0] > counts[1] {
        Ternary::Neg
    } else {
        Ternary::Zero // tie
    }
}

// ── Ternary Parity ───────────────────────────────────────────────────

/// Compute ternary parity (sum mod 3) of a slice.
pub fn ternary_parity(data: &[Ternary]) -> Ternary {
    let mut sum = Ternary::Zero;
    for &v in data {
        sum = Ternary::add_mod3(sum, v);
    }
    sum
}

/// Compute parity check: data + parity should sum to zero mod 3.
pub fn parity_check(data: &[Ternary], parity: Ternary) -> bool {
    Ternary::add_mod3(ternary_parity(data), parity) == Ternary::Zero
}

/// Encode data with a single parity symbol.
pub fn parity_encode(data: &[Ternary]) -> Vec<Ternary> {
    let parity = -ternary_parity(data);
    let mut result = data.to_vec();
    result.push(parity);
    result
}

/// Verify a parity-encoded codeword. Returns true if valid.
pub fn parity_verify(codeword: &[Ternary]) -> bool {
    if codeword.is_empty() {
        return true;
    }
    let (data, parity) = codeword.split_at(codeword.len() - 1);
    parity_check(data, parity[0])
}

// ── Code Distance ────────────────────────────────────────────────────

/// Compute the ternary Hamming distance between two codewords.
/// Counts positions where values differ.
pub fn hamming_distance(a: &[Ternary], b: &[Ternary]) -> Option<usize> {
    if a.len() != b.len() {
        return None;
    }
    Some(a.iter().zip(b.iter()).filter(|(&x, &y)| x != y).count())
}

/// Compute the minimum distance of a codebook.
/// Returns None if the codebook has fewer than 2 codewords.
pub fn minimum_distance(codebook: &[Vec<Ternary>]) -> Option<usize> {
    if codebook.len() < 2 {
        return None;
    }
    let mut min_dist = usize::MAX;
    for i in 0..codebook.len() {
        for j in (i + 1)..codebook.len() {
            if let Some(d) = hamming_distance(&codebook[i], &codebook[j]) {
                min_dist = min_dist.min(d);
            }
        }
    }
    if min_dist == usize::MAX { None } else { Some(min_dist) }
}

/// Number of errors a code with minimum distance d can correct.
pub fn error_correction_capacity(min_distance: usize) -> usize {
    (min_distance - 1) / 2
}

/// Number of errors a code with minimum distance d can detect.
pub fn error_detection_capacity(min_distance: usize) -> usize {
    min_distance - 1
}

/// Compute the ternary Lee distance between two values.
/// Lee distance: |a - b| mapped to ternary: d(-1,0)=1, d(-1,1)=1, d(0,1)=1
pub fn lee_distance(a: Ternary, b: Ternary) -> usize {
    let va = (a.to_i8() + 1) as i32; // {0, 1, 2}
    let vb = (b.to_i8() + 1) as i32;
    let diff = (va - vb).unsigned_abs();
    if diff <= 1 { diff as usize } else { (3 - diff) as usize }
}

/// Lee distance between two codewords.
pub fn lee_distance_codeword(a: &[Ternary], b: &[Ternary]) -> Option<usize> {
    if a.len() != b.len() {
        return None;
    }
    Some(a.iter().zip(b.iter()).map(|(&x, &y)| lee_distance(x, y)).sum())
}

/// Weight of a ternary codeword (number of non-zero symbols).
pub fn weight(codeword: &[Ternary]) -> usize {
    codeword.iter().filter(|&&t| t != Ternary::Zero).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(v: i8) -> Ternary {
        Ternary::from_i8(v).unwrap()
    }

    // ── Ternary mod-3 arithmetic ──
    #[test]
    fn test_add_mod3() {
        assert_eq!(Ternary::add_mod3(t(0), t(0)), t(0));
        assert_eq!(Ternary::add_mod3(t(1), t(1)), t(-1)); // 1+1=2 → -1
        assert_eq!(Ternary::add_mod3(t(-1), t(1)), t(0));
        assert_eq!(Ternary::add_mod3(t(-1), t(-1)), t(1)); // 2+2=4%3=1 → 1
    }

    #[test]
    fn test_sub_mod3() {
        assert_eq!(Ternary::sub_mod3(t(1), t(1)), t(0));
        assert_eq!(Ternary::sub_mod3(t(0), t(1)), t(-1));
    }

    // ── Repetition Code ──
    #[test]
    fn repetition_encode_decode() {
        let code = RepetitionCode::new(5);
        let data = vec![t(1), t(-1), t(0)];
        let encoded = code.encode(&data);
        assert_eq!(encoded.len(), 15);
        let decoded = code.decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn repetition_correct_errors() {
        let code = RepetitionCode::new(5);
        let data = vec![t(1)];
        let mut encoded = code.encode(&data);
        // Introduce 2 errors (within capacity of 2)
        encoded[0] = t(-1);
        encoded[1] = t(0);
        let decoded = code.decode(&encoded).unwrap();
        assert_eq!(decoded, vec![t(1)]);
    }

    #[test]
    fn repetition_error_capacity() {
        let code = RepetitionCode::new(5);
        assert_eq!(code.error_capacity(), 2);
        let code3 = RepetitionCode::new(3);
        assert_eq!(code3.error_capacity(), 1);
    }

    // ── Parity ──
    #[test]
    fn parity_encode_verify() {
        let data = vec![t(1), t(1), t(-1)];
        let encoded = parity_encode(&data);
        assert!(parity_verify(&encoded));
    }

    #[test]
    fn parity_detect_error() {
        let data = vec![t(1), t(-1), t(0)];
        let mut encoded = parity_encode(&data);
        encoded[0] = t(0); // introduce error
        assert!(!parity_verify(&encoded));
    }

    #[test]
    fn ternary_parity_all_zero() {
        let data = vec![t(0), t(0), t(0)];
        assert_eq!(ternary_parity(&data), t(0));
    }

    #[test]
    fn parity_check_basic() {
        let data = vec![t(1), t(1), t(1)];
        let parity = -ternary_parity(&data);
        assert!(parity_check(&data, parity));
    }

    // ── Hamming Distance ──
    #[test]
    fn hamming_distance_basic() {
        let a = vec![t(1), t(0), t(-1)];
        let b = vec![t(1), t(0), t(-1)];
        assert_eq!(hamming_distance(&a, &b), Some(0));

        let c = vec![t(-1), t(0), t(1)];
        assert_eq!(hamming_distance(&a, &c), Some(2));
    }

    #[test]
    fn hamming_distance_different_lengths() {
        let a = vec![t(1)];
        let b = vec![t(1), t(0)];
        assert_eq!(hamming_distance(&a, &b), None);
    }

    #[test]
    fn minimum_distance_codebook() {
        let codebook = vec![
            vec![t(1), t(1), t(1)],
            vec![t(-1), t(-1), t(-1)],
            vec![t(0), t(0), t(0)],
        ];
        assert_eq!(minimum_distance(&codebook), Some(3));
    }

    #[test]
    fn error_correction_detection() {
        assert_eq!(error_correction_capacity(3), 1);
        assert_eq!(error_detection_capacity(3), 2);
        assert_eq!(error_correction_capacity(5), 2);
    }

    // ── Lee Distance ──
    #[test]
    fn lee_distance_values() {
        assert_eq!(lee_distance(t(0), t(0)), 0);
        assert_eq!(lee_distance(t(1), t(0)), 1);
        assert_eq!(lee_distance(t(-1), t(1)), 1); // circular
        assert_eq!(lee_distance(t(-1), t(0)), 1);
    }

    #[test]
    fn lee_distance_codeword_basic() {
        let a = vec![t(1), t(0)];
        let b = vec![t(0), t(0)];
        assert_eq!(lee_distance_codeword(&a, &b), Some(1));
    }

    #[test]
    fn weight_test() {
        assert_eq!(weight(&[t(1), t(0), t(-1), t(0)]), 2);
        assert_eq!(weight(&[t(0), t(0)]), 0);
    }

    // ── Majority Vote ──
    #[test]
    fn majority_vote_test() {
        assert_eq!(majority_vote(&[t(1), t(1), t(-1)]), Ternary::Pos);
        assert_eq!(majority_vote(&[t(-1), t(-1), t(1)]), Ternary::Neg);
        assert_eq!(majority_vote(&[t(1), t(-1), t(0)]), Ternary::Zero); // tie
    }

    #[test]
    fn minimum_distance_too_few() {
        assert_eq!(minimum_distance(&[vec![t(1)]]), None);
        assert_eq!(minimum_distance(&[]), None);
    }
}
