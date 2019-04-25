use std::collections::BTreeMap;

use super::super::arithmetic_coder::{FrequencyTable, SimpleFrequencyTable, Symbol};

pub struct Context {
    pub frequencies: SimpleFrequencyTable,
    pub sub_ctxs: BTreeMap<Symbol, Context>,
}

impl Context {
    pub fn new(num_symbols: u16) -> Self {
        Context {
            frequencies: SimpleFrequencyTable::new(num_symbols),
            sub_ctxs: BTreeMap::new(),
        }
    }
}