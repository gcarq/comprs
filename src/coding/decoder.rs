use bitbit::{BitReader, MSB};
use coding::base::ArithmeticCoderBase;
use coding::FrequencyTable;
use coding::Symbol;
use std::io::{Read, Result};


pub struct ArithmeticDecoder<R: Read> {
    reader: BitReader<R, MSB>,

    // The current raw code bits being buffered,
    // which is always in the range [low, high]
    code: usize,

    low: usize,
    high: usize,
    state_mask: usize,
    full_range: usize,
    half_range: usize,
    quarter_range: usize,
    minimum_range: usize,
    maximum_total: usize,

}

impl<R: Read> ArithmeticCoderBase for ArithmeticDecoder<R> {
    fn set_low(&mut self, value: usize) { self.low = value }
    fn set_high(&mut self, value: usize) { self.high = value }

    fn low(&self) -> usize { self.low }
    fn high(&self) -> usize { self.high }
    fn state_mask(&self) -> usize {self.state_mask }
    fn minimum_range(&self) -> usize { self.minimum_range }
    fn quarter_range(&self) -> usize { self.quarter_range }
    fn half_range(&self) -> usize { self.half_range }
    fn full_range(&self) -> usize { self.full_range }
    fn maximum_total(&self) -> usize { self.maximum_total }

    fn shift(&mut self) -> Result<()> {
        let bit = if self.reader.read_bit().unwrap_or(false) { 1 } else { 0 };
        self.code = ((self.code << 1) & self.state_mask) | bit;
        Ok(())
    }

    fn underflow(&mut self) {
        let bit = if self.reader.read_bit().unwrap_or(false) { 1 } else { 0 };
        self.code = (self.code & self.half_range) | ((self.code << 1) & (self.state_mask >> 1)) | bit;
    }
}

impl<R: Read> ArithmeticDecoder<R> {
    pub fn new(mut reader: BitReader<R, MSB>, num_bits: usize) -> Result<Self> {
        let num_state_bits = num_bits;
        let full_range = 1 << num_state_bits;
        // The top bit at width num_state_bits, which is 0100...000.
        let half_range = full_range >> 1;  // Non-zero
        // The second highest bit at width num_state_bits, which is 0010...000. This is zero when num_state_bits=1.
        let quarter_range = half_range >> 1;  // Can be zero
        // Minimum range (high+1-low) during coding (non-trivial), which is 0010...010.
        let minimum_range = quarter_range + 2;  // At least 2
        // Maximum allowed total from a frequency table at all times during coding. This differs from Java
        // and C++ because Python's native bigint avoids constraining the size of intermediate computations.
        let maximum_total = minimum_range;
        // Bit mask of num_state_bits ones, which is 0111...111.
        let state_mask = full_range - 1;

        // Low end of this arithmetic coder's current range. Conceptually has an infinite number of trailing 0s.
        let low = 0;
        // High end of this arithmetic coder's current range. Conceptually has an infinite number of trailing 1s.
        let high = state_mask;

        let mut code = 0;
        for _ in 0..num_bits {
            let bit = if reader.read_bit()? { 1 } else { 0 };
            code = code << 1 | bit;
        }

        Ok(ArithmeticDecoder {
            reader,
            low, high, state_mask,
            full_range, half_range, quarter_range,
            minimum_range, maximum_total,
            code
        })
    }

    pub fn read<T: FrequencyTable>(&mut self, freqtable: &mut T) -> Result<Symbol> {
        let total = freqtable.total();

        debug_assert!(total <= self.maximum_total);

        let range = self.high - self.low + 1;
        let offset = self.code - self.low;
        let value = ((offset + 1) * total - 1) / range;
        debug_assert!(value * range / total <= offset);
        debug_assert!(value < total);

        // A kind of binary search. Find highest symbol such that freqs.get_low(symbol) <= value.
        let mut start: u16 = 0;
        let mut end: u16 = freqtable.get_symbol_limit();
        while end - start > 1 {
            let middle: u16 = (start + end) >> 1;
            if freqtable.get_low(middle) > value {
                end = middle;
            } else {
                start = middle;
            }
        }
        debug_assert_eq!(start + 1, end);

        let symbol: Symbol = start;
        debug_assert!(freqtable.get_low(symbol) * range / total <= offset);
        debug_assert!(offset < freqtable.get_high(symbol) * range / total);
        self.update(freqtable, symbol)?;

        debug_assert!(self.low <= self.code);
        debug_assert!(self.code <= self.high);

        Ok(symbol)
    }
}
