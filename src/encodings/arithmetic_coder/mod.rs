use std::fmt;

mod base;
pub mod encoder;
pub mod decoder;

pub type Symbol = u16;

pub trait FrequencyTable {
    fn new(num_symbols: u16) -> Self;
    fn get(&self, symbol: Symbol) -> usize;
    fn get_low(&mut self, symbol: Symbol) -> usize;
    fn get_high(&mut self, symbol: Symbol) -> usize;
    fn get_symbol_limit(&self) -> u16;
    fn set(&mut self, symbol: Symbol, frequency: usize);
    fn increment(&mut self, symbol: Symbol);
    fn total(&self) -> usize;
}


pub struct SimpleFrequencyTable {
    pub frequencies: Vec<usize>,
    pub total: usize,
}

impl SimpleFrequencyTable {
    fn cumulative(&mut self, symbol: Symbol) -> usize {
        self.frequencies.iter().take(symbol as usize).sum()
    }
}

impl FrequencyTable for SimpleFrequencyTable {

    fn new(num_symbols: u16) -> Self {
        SimpleFrequencyTable {
            frequencies: vec![0; num_symbols as usize],
            total: 0,
        }
    }

    fn get(&self, symbol: Symbol) -> usize {
        self.frequencies[symbol as usize]
    }

    fn get_low(&mut self, symbol: Symbol) -> usize {
        self.cumulative(symbol)
    }

    fn get_high(&mut self, symbol: Symbol) -> usize {
        self.cumulative(symbol + 1)
    }

    fn get_symbol_limit(&self) -> u16 {
        self.frequencies.len() as u16
    }

    fn set(&mut self, symbol: Symbol, frequency: usize) {
        self.total -= self.frequencies[symbol as usize];
        self.frequencies[symbol as usize] = frequency;
        self.total += frequency;
    }

    fn increment(&mut self, symbol: Symbol) {
        self.total += 1;
        self.frequencies[symbol as usize] += 1;
    }

    fn total(&self) -> usize { self.total }
}

impl fmt::Debug for SimpleFrequencyTable {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("SimpleFrequencyTable")
           .field("frequencies", &self.frequencies)
           .finish()
    }
}

pub struct FlatFrequencyTable {
    num_symbols: u16
}

impl FrequencyTable for FlatFrequencyTable {
    fn new(num_symbols: u16) -> Self {
        FlatFrequencyTable{ num_symbols }
    }

    fn get(&self, _symbol: Symbol) -> usize { 1 }

    fn get_low(&mut self, symbol: Symbol) -> usize { symbol as usize }

    fn get_high(&mut self, symbol: Symbol) -> usize { symbol as usize + 1 }

    fn get_symbol_limit(&self) -> Symbol { self.num_symbols }

    fn set(&mut self, _symbol: Symbol, _frequency: usize) {
        unimplemented!()
    }

    fn increment(&mut self, _symbol: Symbol) {
        unimplemented!()
    }

    fn total(&self) -> usize { self.num_symbols as usize }
}