use std::io::Result;

use crate::encodings::arithmetic_coder::Symbol;

use super::FrequencyTable;

pub trait ArithmeticCoderBase {
    fn set_low(&mut self, value: usize);
    fn set_high(&mut self, value: usize);

    fn low(&self) -> usize;
    fn high(&self) -> usize;
    fn state_mask(&self) -> usize;
    fn minimum_range(&self) -> usize;
    fn quarter_range(&self) -> usize;
    fn half_range(&self) -> usize;
    fn full_range(&self) -> usize;
    fn maximum_total(&self) -> usize;

    fn shift(&mut self) -> Result<()>;
    fn underflow(&mut self);

    fn update<T: FrequencyTable>(&mut self, freqtable: &mut T, symbol: Symbol) -> Result<()> {
        let (low, high) = (self.low(), self.high());
        debug_assert!(low < high, "low or high out of range");

        debug_assert!(low & self.state_mask() == low, "low out of range");
        debug_assert!(high & self.state_mask() == high, "high out of range");

        let range = high - low + 1;
        debug_assert!(self.minimum_range() <= range);
        debug_assert!(range <= self.full_range());

        let symlow = freqtable.get_low(symbol);
        let symhigh = freqtable.get_high(symbol);
        let total = freqtable.total();
        debug_assert!(symlow != symhigh, "symbol has zero frequency");
        debug_assert!(
            total <= self.maximum_total(),
            "cannot code symbol because total is too large"
        );

        let (mut low, mut high) = (
            low + symlow * range / total,
            low + symhigh * range / total - 1,
        );

        // While low and high have the same top bit value, shift them out
        let half_range = self.half_range();
        let state_mask = self.state_mask();
        while ((low ^ high) & half_range) == 0 {
            // shift() needs an updated low value
            self.set_low(low);
            self.shift()?;
            low = (low << 1) & state_mask;
            high = ((high << 1) & state_mask) | 1;
        }
        // Now low's top bit must be 0 and high's top bit must be 1

        // While low's top two bits are 01 and high's are 10, delete the second highest bit of both
        let quarter_range = self.quarter_range();
        while (low & !high & quarter_range) != 0 {
            self.underflow();
            low = (low << 1) ^ half_range;
            high = ((high ^ half_range) << 1) | half_range | 1;
        }
        self.set_low(low);
        self.set_high(high);
        Ok(())
    }
}
