use super::context::Context;
use super::super::arithmetic_coder::{FlatFrequencyTable, FrequencyTable, Symbol};

pub struct PPMModel {
    pub context: Context,
    pub order: u8,
    pub order_minus1_freqs: FlatFrequencyTable,
    symbol_limit: u16,
    escape_symbol: Symbol,
}

impl PPMModel {
    pub fn new(order: u8, symbol_limit: u16, escape_symbol: Symbol) -> Self {
        debug_assert!(escape_symbol < symbol_limit);

        let mut context = Context::new(symbol_limit);
        context.frequencies.increment(escape_symbol);
        PPMModel {
            order_minus1_freqs: FlatFrequencyTable::new(symbol_limit),
            order, symbol_limit, escape_symbol, context,
        }
    }

    pub fn increment_contexts(&mut self, history: &[Symbol], symbol: Symbol) {
        let hist_len = history.len();

        debug_assert!(hist_len <= self.order as usize);
        debug_assert!(symbol < self.symbol_limit);

        for order in 0..=hist_len {
            populate_contexts(&mut self.context, &history[hist_len-order..hist_len], symbol, self.escape_symbol, self.symbol_limit);
        }
    }
}

fn populate_contexts(ctx: &mut Context, history: &[Symbol], symbol: Symbol, escape_symbol: Symbol, symbol_limit: u16) {
    if history.is_empty() {
        ctx.frequencies.increment(symbol);
        return;
    }

    let sym = history[0];
    if ctx.sub_ctxs.get(&sym).is_none() {
        let mut sub_ctx = Context::new(symbol_limit);
        sub_ctx.frequencies.increment(escape_symbol);
        ctx.sub_ctxs.insert(sym, sub_ctx);
    }

    populate_contexts(
        ctx.sub_ctxs.get_mut(&sym).unwrap(),
        &history[1..], symbol, escape_symbol, symbol_limit
    );
}
