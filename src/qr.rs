//! QR codes: encode text into a module matrix and draw it as SVG.
//!
//! Byte mode (UTF-8), versions 1 to 40, all four error correction levels,
//! and the mask with the lowest penalty score. The output is deterministic:
//! the same text and level always give the same matrix and the same SVG.
//!
//! This module stands on its own; it does not use the chart specification.

use std::fmt::{self, Write as _};

/// How much of the code may be damaged and still be read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ErrorCorrection {
    /// About 7 %.
    Low,
    /// About 15 %.
    #[default]
    Medium,
    /// About 25 %.
    Quartile,
    /// About 30 %.
    High,
}

impl ErrorCorrection {
    fn index(self) -> usize {
        match self {
            Self::Low => 0,
            Self::Medium => 1,
            Self::Quartile => 2,
            Self::High => 3,
        }
    }

    /// The two level bits of the format information.
    fn format_bits(self) -> u32 {
        match self {
            Self::Low => 1,
            Self::Medium => 0,
            Self::Quartile => 3,
            Self::High => 2,
        }
    }
}

/// The data does not fit into the largest QR code (version 40) at the
/// requested error correction level.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataTooLong {
    /// Length of the data in bytes.
    pub bytes: usize,
    /// Largest length that fits at this level.
    pub max_bytes: usize,
}

impl fmt::Display for DataTooLong {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} bytes do not fit into a QR code at this error correction level (at most {})",
            self.bytes, self.max_bytes
        )
    }
}

impl std::error::Error for DataTooLong {}

const MIN_VERSION: u8 = 1;
const MAX_VERSION: u8 = 40;

/// Error correction codewords per block, by level and version (index 0 unused).
const ECC_CODEWORDS_PER_BLOCK: [[u8; 41]; 4] = [
    [
        0, 7, 10, 15, 20, 26, 18, 20, 24, 30, 18, 20, 24, 26, 30, 22, 24, 28, 30, 28, 28, 28, 28,
        30, 30, 26, 28, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30,
    ],
    [
        0, 10, 16, 26, 18, 24, 16, 18, 22, 22, 26, 30, 22, 22, 24, 24, 28, 28, 26, 26, 26, 26, 28,
        28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28,
    ],
    [
        0, 13, 22, 18, 26, 18, 24, 18, 22, 20, 24, 28, 26, 24, 20, 30, 24, 28, 28, 26, 30, 28, 30,
        30, 30, 30, 28, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30,
    ],
    [
        0, 17, 28, 22, 16, 22, 28, 26, 26, 24, 28, 24, 28, 22, 24, 24, 30, 28, 28, 26, 28, 30, 24,
        30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30,
    ],
];

/// Error correction blocks, by level and version (index 0 unused).
const ERROR_CORRECTION_BLOCKS: [[u8; 41]; 4] = [
    [
        0, 1, 1, 1, 1, 1, 2, 2, 2, 2, 4, 4, 4, 4, 4, 6, 6, 6, 6, 7, 8, 8, 9, 9, 10, 12, 12, 12, 13,
        14, 15, 16, 17, 18, 19, 19, 20, 21, 22, 24, 25,
    ],
    [
        0, 1, 1, 1, 2, 2, 4, 4, 4, 5, 5, 5, 8, 9, 9, 10, 10, 11, 13, 14, 16, 17, 17, 18, 20, 21,
        23, 25, 26, 28, 29, 31, 33, 35, 37, 38, 40, 43, 45, 47, 49,
    ],
    [
        0, 1, 1, 2, 2, 4, 4, 6, 6, 8, 8, 8, 10, 12, 16, 12, 17, 16, 18, 21, 20, 23, 23, 25, 27, 29,
        34, 34, 35, 38, 40, 43, 45, 48, 51, 53, 56, 59, 62, 65, 68,
    ],
    [
        0, 1, 1, 2, 4, 4, 4, 5, 6, 8, 8, 11, 11, 16, 16, 18, 16, 19, 21, 25, 25, 25, 34, 30, 32,
        35, 37, 40, 42, 45, 48, 51, 54, 57, 60, 63, 66, 70, 74, 77, 81,
    ],
];

/// Penalty weights for mask selection.
const PENALTY_RUN: usize = 3;
const PENALTY_BLOCK: usize = 3;
const PENALTY_FINDER: usize = 40;
const PENALTY_BALANCE: usize = 10;

/// A finished QR code: `size` × `size` modules.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QrCode {
    version: u8,
    size: usize,
    modules: Vec<bool>,
}

impl QrCode {
    /// Encodes `text` as UTF-8 bytes in the smallest version that fits.
    ///
    /// # Errors
    ///
    /// Returns [`DataTooLong`] when the text does not fit into version 40.
    pub fn encode(text: &str, level: ErrorCorrection) -> Result<Self, DataTooLong> {
        Self::encode_bytes(text.as_bytes(), level)
    }

    /// Encodes raw bytes in the smallest version that fits.
    ///
    /// # Errors
    ///
    /// Returns [`DataTooLong`] when the data does not fit into version 40.
    pub fn encode_bytes(data: &[u8], level: ErrorCorrection) -> Result<Self, DataTooLong> {
        let version = (MIN_VERSION..=MAX_VERSION)
            .find(|&version| data.len() <= max_bytes(version, level))
            .ok_or(DataTooLong {
                bytes: data.len(),
                max_bytes: max_bytes(MAX_VERSION, level),
            })?;
        let codewords = add_error_correction(&data_codewords(data, version, level), version, level);
        Ok(Self::draw(version, level, &codewords))
    }

    /// The version, 1 to 40.
    #[must_use]
    pub fn version(&self) -> u8 {
        self.version
    }

    /// Modules per side: 17 + 4 × version.
    #[must_use]
    pub fn size(&self) -> usize {
        self.size
    }

    /// Whether the module in column `x`, row `y` is dark; false outside the code.
    #[must_use]
    pub fn is_dark(&self, x: usize, y: usize) -> bool {
        x < self.size && y < self.size && self.modules[y * self.size + x]
    }

    /// The code as SVG: dark modules on a light background with a quiet
    /// zone of `quiet_zone` modules on every side, one unit per module,
    /// `role="img"` with `title` as its accessible name.
    #[must_use]
    pub fn to_svg(&self, title: &str, quiet_zone: usize) -> String {
        let side = self.size + 2 * quiet_zone;
        let mut path = String::new();
        for y in 0..self.size {
            let mut x = 0;
            while x < self.size {
                if !self.is_dark(x, y) {
                    x += 1;
                    continue;
                }
                let start = x;
                while x < self.size && self.is_dark(x, y) {
                    x += 1;
                }
                let _ = write!(
                    path,
                    "M{},{}h{}v1h-{}z",
                    start + quiet_zone,
                    y + quiet_zone,
                    x - start,
                    x - start
                );
            }
        }
        format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {side} {side}\" width=\"{w}\" height=\"{w}\" role=\"img\" shape-rendering=\"crispEdges\"><title>{title}</title><rect width=\"{side}\" height=\"{side}\" fill=\"#fff\"/><path d=\"{path}\" fill=\"#000\"/></svg>",
            w = side * 4,
            title = escape(title),
        )
    }

    /// Places the function patterns and the codewords, then applies the
    /// mask with the lowest penalty.
    fn draw(version: u8, level: ErrorCorrection, codewords: &[u8]) -> Self {
        let size = usize::from(version) * 4 + 17;
        let mut canvas = Canvas {
            size,
            modules: vec![false; size * size],
            function: vec![false; size * size],
        };
        canvas.draw_function_patterns(version);
        canvas.draw_format_bits(level, 0);
        canvas.draw_codewords(codewords);

        let mut best = (usize::MAX, 0);
        for mask in 0..8 {
            canvas.apply_mask(mask);
            canvas.draw_format_bits(level, mask);
            let penalty = canvas.penalty();
            if penalty < best.0 {
                best = (penalty, mask);
            }
            canvas.apply_mask(mask);
        }
        canvas.apply_mask(best.1);
        canvas.draw_format_bits(level, best.1);
        Self {
            version,
            size,
            modules: canvas.modules,
        }
    }
}

fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
    out
}

/// Data modules of a version: everything except function patterns and
/// format/version information.
fn raw_data_modules(version: u8) -> usize {
    let v = usize::from(version);
    let mut result = (16 * v + 128) * v + 64;
    if v >= 2 {
        let alignments = v / 7 + 2;
        result -= (25 * alignments - 10) * alignments - 55;
        if v >= 7 {
            result -= 36;
        }
    }
    result
}

fn data_codeword_count(version: u8, level: ErrorCorrection) -> usize {
    let v = usize::from(version);
    let l = level.index();
    raw_data_modules(version) / 8
        - usize::from(ECC_CODEWORDS_PER_BLOCK[l][v]) * usize::from(ERROR_CORRECTION_BLOCKS[l][v])
}

fn count_bits(version: u8) -> usize {
    if version < 10 { 8 } else { 16 }
}

/// Longest byte-mode data that fits.
fn max_bytes(version: u8, level: ErrorCorrection) -> usize {
    (data_codeword_count(version, level) * 8 - 4 - count_bits(version)) / 8
}

fn push_bits(bits: &mut Vec<bool>, value: usize, count: usize) {
    bits.extend((0..count).rev().map(|i| (value >> i) & 1 == 1));
}

/// Mode, length, data, terminator and padding as codewords.
fn data_codewords(data: &[u8], version: u8, level: ErrorCorrection) -> Vec<u8> {
    let capacity = data_codeword_count(version, level) * 8;
    let mut bits = Vec::with_capacity(capacity);
    push_bits(&mut bits, 0b0100, 4);
    push_bits(&mut bits, data.len(), count_bits(version));
    for &byte in data {
        push_bits(&mut bits, usize::from(byte), 8);
    }
    let terminator = (capacity - bits.len()).min(4);
    push_bits(&mut bits, 0, terminator);
    let to_byte = (8 - bits.len() % 8) % 8;
    push_bits(&mut bits, 0, to_byte);
    for pad in [0xEC, 0x11].into_iter().cycle() {
        if bits.len() >= capacity {
            break;
        }
        push_bits(&mut bits, pad, 8);
    }
    bits.chunks(8)
        .map(|chunk| {
            chunk
                .iter()
                .fold(0u8, |byte, &bit| (byte << 1) | u8::from(bit))
        })
        .collect()
}

/// Multiplication in GF(2^8) modulo x^8 + x^4 + x^3 + x^2 + 1.
fn gf_multiply(x: u8, y: u8) -> u8 {
    let mut z = 0u8;
    for i in (0..8).rev() {
        let carry = z & 0x80 != 0;
        z <<= 1;
        if carry {
            z ^= 0x1D;
        }
        if (y >> i) & 1 == 1 {
            z ^= x;
        }
    }
    z
}

/// Coefficients of the Reed–Solomon generator polynomial of `degree`,
/// highest power first, without the leading 1.
fn rs_divisor(degree: usize) -> Vec<u8> {
    let mut result = vec![0u8; degree];
    result[degree - 1] = 1;
    let mut root = 1u8;
    for _ in 0..degree {
        for j in 0..degree {
            result[j] = gf_multiply(result[j], root);
            if j + 1 < degree {
                result[j] ^= result[j + 1];
            }
        }
        root = gf_multiply(root, 0x02);
    }
    result
}

fn rs_remainder(data: &[u8], divisor: &[u8]) -> Vec<u8> {
    let mut result = vec![0u8; divisor.len()];
    for &byte in data {
        let factor = byte ^ result.remove(0);
        result.push(0);
        for (coefficient, &d) in result.iter_mut().zip(divisor) {
            *coefficient ^= gf_multiply(d, factor);
        }
    }
    result
}

/// Splits the data into blocks, appends each block's error correction and
/// interleaves the result.
fn add_error_correction(data: &[u8], version: u8, level: ErrorCorrection) -> Vec<u8> {
    let v = usize::from(version);
    let block_count = usize::from(ERROR_CORRECTION_BLOCKS[level.index()][v]);
    let ecc_length = usize::from(ECC_CODEWORDS_PER_BLOCK[level.index()][v]);
    let raw_codewords = raw_data_modules(version) / 8;
    let short_blocks = block_count - raw_codewords % block_count;
    let short_length = raw_codewords / block_count;
    let divisor = rs_divisor(ecc_length);

    let mut blocks: Vec<Vec<u8>> = Vec::with_capacity(block_count);
    let mut offset = 0;
    for i in 0..block_count {
        let length = short_length - ecc_length + usize::from(i >= short_blocks);
        let mut block = data[offset..offset + length].to_vec();
        offset += length;
        let ecc = rs_remainder(&block, &divisor);
        if i < short_blocks {
            // Placeholder so all blocks have the same length; skipped below.
            block.push(0);
        }
        block.extend_from_slice(&ecc);
        blocks.push(block);
    }

    let mut result = Vec::with_capacity(raw_codewords);
    for i in 0..blocks[0].len() {
        for (j, block) in blocks.iter().enumerate() {
            if i != short_length - ecc_length || j >= short_blocks {
                result.push(block[i]);
            }
        }
    }
    result
}

/// Centers of the alignment patterns along one axis.
fn alignment_positions(version: u8) -> Vec<usize> {
    if version == 1 {
        return Vec::new();
    }
    let v = usize::from(version);
    let count = v / 7 + 2;
    let step = if v == 32 {
        26
    } else {
        (v * 4 + count * 2 + 1) / (count * 2 - 2) * 2
    };
    let size = v * 4 + 17;
    let mut result = vec![6];
    let mut position = size - 7;
    for _ in 0..count - 1 {
        result.insert(1, position);
        position -= step;
    }
    result
}

fn bit(value: u32, index: u32) -> bool {
    (value >> index) & 1 == 1
}

struct Canvas {
    size: usize,
    modules: Vec<bool>,
    /// Modules of function patterns, which the data and the mask skip.
    function: Vec<bool>,
}

impl Canvas {
    fn get(&self, x: usize, y: usize) -> bool {
        self.modules[y * self.size + x]
    }

    fn set_function(&mut self, x: usize, y: usize, dark: bool) {
        self.modules[y * self.size + x] = dark;
        self.function[y * self.size + x] = true;
    }

    fn draw_function_patterns(&mut self, version: u8) {
        let size = self.size;
        for i in 0..size {
            self.set_function(6, i, i % 2 == 0);
            self.set_function(i, 6, i % 2 == 0);
        }
        self.draw_finder(3, 3);
        self.draw_finder(size - 4, 3);
        self.draw_finder(3, size - 4);

        let positions = alignment_positions(version);
        let last = positions.len().saturating_sub(1);
        for (i, &x) in positions.iter().enumerate() {
            for (j, &y) in positions.iter().enumerate() {
                // The three corners with finder patterns get none.
                let corner = (i == 0 && (j == 0 || j == last)) || (i == last && j == 0);
                if !corner {
                    self.draw_alignment(x, y);
                }
            }
        }
        self.draw_version(version);
    }

    /// Finder pattern with its light separator, centered on (`x`, `y`).
    fn draw_finder(&mut self, x: usize, y: usize) {
        for dy in 0..9 {
            for dx in 0..9 {
                let (Some(xx), Some(yy)) = ((x + dx).checked_sub(4), (y + dy).checked_sub(4))
                else {
                    continue;
                };
                if xx < self.size && yy < self.size {
                    let distance = dx.abs_diff(4).max(dy.abs_diff(4));
                    self.set_function(xx, yy, distance != 2 && distance != 4);
                }
            }
        }
    }

    fn draw_alignment(&mut self, x: usize, y: usize) {
        for dy in 0usize..5 {
            for dx in 0usize..5 {
                let distance = dx.abs_diff(2).max(dy.abs_diff(2));
                self.set_function(x + dx - 2, y + dy - 2, distance != 1);
            }
        }
    }

    /// Format information (level and mask, BCH-protected) in both copies,
    /// plus the dark module.
    fn draw_format_bits(&mut self, level: ErrorCorrection, mask: u32) {
        let data = (level.format_bits() << 3) | mask;
        let mut remainder = data;
        for _ in 0..10 {
            remainder = (remainder << 1) ^ ((remainder >> 9) * 0x537);
        }
        let bits = ((data << 10) | remainder) ^ 0x5412;
        let size = self.size;

        for i in 0..6 {
            self.set_function(8, i as usize, bit(bits, i));
        }
        self.set_function(8, 7, bit(bits, 6));
        self.set_function(8, 8, bit(bits, 7));
        self.set_function(7, 8, bit(bits, 8));
        for i in 9..15 {
            self.set_function(14 - i as usize, 8, bit(bits, i));
        }

        for i in 0..8 {
            self.set_function(size - 1 - i as usize, 8, bit(bits, i));
        }
        for i in 8..15 {
            self.set_function(8, size - 15 + i as usize, bit(bits, i));
        }
        self.set_function(8, size - 8, true);
    }

    /// Version information (BCH-protected) for versions 7 and up.
    fn draw_version(&mut self, version: u8) {
        if version < 7 {
            return;
        }
        let data = u32::from(version);
        let mut remainder = data;
        for _ in 0..12 {
            remainder = (remainder << 1) ^ ((remainder >> 11) * 0x1F25);
        }
        let bits = (data << 12) | remainder;
        for i in 0..18 {
            let dark = bit(bits, i);
            let a = self.size - 11 + i as usize % 3;
            let b = i as usize / 3;
            self.set_function(a, b, dark);
            self.set_function(b, a, dark);
        }
    }

    /// Places the codeword bits in the zigzag order: two-module columns
    /// from the right, alternately upwards and downwards, skipping the
    /// vertical timing pattern.
    fn draw_codewords(&mut self, codewords: &[u8]) {
        let size = self.size;
        let total_bits = codewords.len() * 8;
        let mut index = 0;
        let mut right = size - 1;
        loop {
            if right == 6 {
                right = 5;
            }
            for vertical in 0..size {
                for j in 0..2 {
                    let x = right - j;
                    let upwards = (right + 1) & 2 == 0;
                    let y = if upwards {
                        size - 1 - vertical
                    } else {
                        vertical
                    };
                    if !self.function[y * size + x] && index < total_bits {
                        self.modules[y * size + x] =
                            (codewords[index / 8] >> (7 - index % 8)) & 1 == 1;
                        index += 1;
                    }
                }
            }
            if right < 2 {
                break;
            }
            right -= 2;
        }
    }

    /// XORs the data modules with mask pattern `mask`; applying it twice undoes it.
    fn apply_mask(&mut self, mask: u32) {
        let size = self.size;
        for y in 0..size {
            for x in 0..size {
                let invert = match mask {
                    0 => (x + y) % 2 == 0,
                    1 => y % 2 == 0,
                    2 => x % 3 == 0,
                    3 => (x + y) % 3 == 0,
                    4 => (x / 3 + y / 2) % 2 == 0,
                    5 => x * y % 2 + x * y % 3 == 0,
                    6 => (x * y % 2 + x * y % 3) % 2 == 0,
                    _ => ((x + y) % 2 + x * y % 3) % 2 == 0,
                };
                if invert && !self.function[y * size + x] {
                    self.modules[y * size + x] ^= true;
                }
            }
        }
    }

    /// Penalty score used to pick the mask: long runs, 2×2 blocks,
    /// finder-like patterns and the dark/light balance.
    fn penalty(&self) -> usize {
        let size = self.size;
        let mut result = 0;
        for transposed in [false, true] {
            for a in 0..size {
                let mut run_color = false;
                let mut run_length = 0;
                let mut history = [0usize; 7];
                for b in 0..size {
                    let color = if transposed {
                        self.get(a, b)
                    } else {
                        self.get(b, a)
                    };
                    if color == run_color {
                        run_length += 1;
                        if run_length == 5 {
                            result += PENALTY_RUN;
                        } else if run_length > 5 {
                            result += 1;
                        }
                    } else {
                        add_history(run_length, &mut history, size);
                        if !run_color {
                            result += finder_patterns(&history) * PENALTY_FINDER;
                        }
                        run_color = color;
                        run_length = 1;
                    }
                }
                if run_color {
                    add_history(run_length, &mut history, size);
                    run_length = 0;
                }
                add_history(run_length + size, &mut history, size);
                result += finder_patterns(&history) * PENALTY_FINDER;
            }
        }
        for y in 0..size - 1 {
            for x in 0..size - 1 {
                let color = self.get(x, y);
                if color == self.get(x + 1, y)
                    && color == self.get(x, y + 1)
                    && color == self.get(x + 1, y + 1)
                {
                    result += PENALTY_BLOCK;
                }
            }
        }
        let dark = self.modules.iter().filter(|&&m| m).count();
        let total = size * size;
        let k = (dark * 20).abs_diff(total * 10).div_ceil(total) - 1;
        result + k * PENALTY_BALANCE
    }
}

/// Pushes a run length; the first run of a line also counts the light
/// border around the code.
fn add_history(mut run_length: usize, history: &mut [usize; 7], size: usize) {
    if history[0] == 0 {
        run_length += size;
    }
    history.copy_within(0..6, 1);
    history[0] = run_length;
}

/// Finder-like 1:1:3:1:1 patterns with four light modules on one side.
fn finder_patterns(history: &[usize; 7]) -> usize {
    let n = history[1];
    let core =
        n > 0 && history[2] == n && history[3] == n * 3 && history[4] == n && history[5] == n;
    usize::from(core && history[0] >= n * 4 && history[6] >= n)
        + usize::from(core && history[6] >= n * 4 && history[0] >= n)
}
