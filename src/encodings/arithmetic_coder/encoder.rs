use std::cmp;
use std::io::{Result, Write};

use bitbit::BitWriter;

use crate::encodings::arithmetic_coder::Symbol;

use super::base::ArithmeticCoderBase;
use super::FrequencyTable;

pub struct ArithmeticEncoder<W: Write> {
    writer: BitWriter<W>,
    num_underflow: usize,
    num_state_bits: usize,

    low: usize,
    high: usize,
    state_mask: usize,
    full_range: usize,
    half_range: usize,
    quarter_range: usize,
    minimum_range: usize,
    maximum_total: usize,
}

impl<W: Write> ArithmeticEncoder<W> {
    pub fn new(writer: BitWriter<W>, num_bits: usize) -> Self {
        debug_assert!(num_bits > 0);
        debug_assert!(num_bits < 64);
        let num_state_bits = num_bits;
        let full_range = 1 << num_state_bits;
        // The top bit at width num_state_bits, which is 0100...000.
        let half_range = full_range >> 1; // Non-zero
                                          // The second highest bit at width num_state_bits, which is 0010...000. This is zero when num_state_bits=1.
        let quarter_range = half_range >> 1; // Can be zero
                                             // Minimum range (high+1-low) during coding (non-trivial), which is 0010...010.
        let minimum_range = quarter_range + 2; // At least 2
                                               // Maximum allowed total from a frequency table at all times during coding. This differs from Java
                                               // and C++ because Python's native bigint avoids constraining the size of intermediate computations.
        let maximum_total = cmp::min(std::usize::MAX / full_range, minimum_range);
        // Bit mask of num_state_bits ones, which is 0111...111.
        let state_mask = full_range - 1;

        // Low end of this arithmetic coder's current range. Conceptually has an infinite number of trailing 0s.
        let low = 0;
        // High end of this arithmetic coder's current range. Conceptually has an infinite number of trailing 1s.
        let high = state_mask;

        let num_underflow = 0;

        ArithmeticEncoder {
            writer,
            num_state_bits,
            num_underflow,
            low,
            high,
            state_mask,
            full_range,
            half_range,
            quarter_range,
            minimum_range,
            maximum_total,
        }
    }
    #[inline]
    pub fn write<T: FrequencyTable>(&mut self, freqtable: &mut T, symbol: Symbol) -> Result<()> {
        self.update(freqtable, symbol)
    }

    /// Terminates the arithmetic coding by flushing any buffered bits, so that the output can be decoded properly.
    /// It is important that this method must be called at the end of the each encoding process.
    /// Note that this method merely writes data to the underlying output stream but does not close it.
    pub fn finish(&mut self) -> Result<()> {
        self.writer.write_bit(true)?;
        self.writer.write_byte(0)
    }

    /// Get reference of the inner writer
    #[inline]
    pub fn inner_ref(&mut self) -> &W {
        self.writer.get_ref()
    }
}

impl<W: Write> ArithmeticCoderBase for ArithmeticEncoder<W> {
    fn set_low(&mut self, value: usize) {
        self.low = value
    }
    fn set_high(&mut self, value: usize) {
        self.high = value
    }
    #[inline]
    fn low(&self) -> usize {
        self.low
    }
    #[inline]
    fn high(&self) -> usize {
        self.high
    }
    #[inline]
    fn state_mask(&self) -> usize {
        self.state_mask
    }
    #[inline]
    fn minimum_range(&self) -> usize {
        self.minimum_range
    }
    #[inline]
    fn quarter_range(&self) -> usize {
        self.quarter_range
    }
    #[inline]
    fn half_range(&self) -> usize {
        self.half_range
    }
    #[inline]
    fn full_range(&self) -> usize {
        self.full_range
    }
    #[inline]
    fn maximum_total(&self) -> usize {
        self.maximum_total
    }

    fn shift(&mut self) -> Result<()> {
        let bit = match self.low >> (self.num_state_bits - 1) {
            1 => true,
            0 => false,
            _ => panic!("shift overflow"),
        };
        self.writer.write_bit(bit)?;

        // Write out the saved underflow bits
        for _ in 0..self.num_underflow {
            self.writer.write_bit(!bit)?;
        }
        self.num_underflow = 0;
        Ok(())
    }

    fn underflow(&mut self) {
        self.num_underflow += 1;
    }
}
