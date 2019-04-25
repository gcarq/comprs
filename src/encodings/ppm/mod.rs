use std::io::{Read, Result, Write};

use bitbit::{BitReader, BitWriter};

use super::arithmetic_coder::{FrequencyTable, Symbol};
use super::arithmetic_coder::decoder::ArithmeticDecoder;
use super::arithmetic_coder::encoder::ArithmeticEncoder;

use self::context::Context;
use self::model::PPMModel;

pub mod context;
pub mod model;

// TODO: create struct to hold all possible encoding parameters
const EOF: Symbol = 256;
const ORDER: u8 = 2;
const SYMBOL_LIMIT: u16 = 257;
const ESCAPE_SYMBOL: Symbol = 256;
const NUM_BITS: usize = 32;


/// Compress content provided by reader and write compressed data to writer.
pub fn apply(data: &[u8]) -> Result<Vec<u8>> {
    let mut encoder = ArithmeticEncoder::new(
        BitWriter::new(Vec::with_capacity(data.len() / 4)),
        NUM_BITS);
    let mut model = PPMModel::new(ORDER as u8, SYMBOL_LIMIT, ESCAPE_SYMBOL);
    let mut history: Vec<Symbol> = Vec::with_capacity(model.order as usize);

    for byte in data {
        let symbol = u16::from(*byte);
        encode_symbol(&mut model, &history, symbol, &mut encoder)?;
        model.increment_contexts(&history, symbol);

        if model.order >= 1 {
            mutate_history(symbol, model.order, &mut history);
        }
    }

    // Encode EOF
    encode_symbol(&mut model, &history, EOF, &mut encoder)?;
    encoder.finish()?;
    Ok(encoder.inner_ref().clone())
}

/// Decompress content provided by reader and write restored data to writer.
pub fn reduce(data: &[u8]) -> Result<Vec<u8>> {
    let mut decoder = ArithmeticDecoder::new(BitReader::new(data), NUM_BITS)?;
    let mut model = PPMModel::new(ORDER as u8, SYMBOL_LIMIT, ESCAPE_SYMBOL);

    let mut history: Vec<u16> = Vec::with_capacity(model.order as usize);
    let mut buffer = Vec::with_capacity(data.len());

    loop {
        let symbol = decode_symbol(&mut model, &history, &mut decoder)?;
        // Check if EOF symbol has occurred
        if symbol == EOF {
            break
        }
        buffer.write_all(&[symbol as u8])?;
        model.increment_contexts(&history, symbol);

        if model.order >= 1 {
            mutate_history(symbol, model.order, &mut history);
        }
    }
    Ok(buffer)
}


/// Append current symbol to history or shift back by one
fn mutate_history(symbol: Symbol, order: u8, history: &mut Vec<Symbol>) {
    if history.len() >= order as usize {
        history.remove(0);
    }
    history.push(symbol);
}

/// Return highest order context that exists for the given history prefix.
fn traverse_context<'a>(ctx: &'a mut Context, history: &[Symbol]) -> Option<&'a mut Context> {
    if history.is_empty() {
        return Some(ctx);
    }

    match ctx.sub_ctxs.get_mut(&history[0]) {
        None => None,
        Some(sub) => { traverse_context(sub, &history[1..]) }
    }
}

/// Try to use highest order context that exists based on the history suffix, such
/// that the next symbol has non-zero frequency. When symbol 256 is produced at a context
/// at any non-negative order, it means "escape to the next lower order with non-empty
/// context". When symbol 256 is produced at the order -1 context, it means "EOF".
fn encode_symbol<'a, W: Write>(model: &'a mut PPMModel, history: &[Symbol], symbol: Symbol, encoder: &mut ArithmeticEncoder<W>) -> Result<()> {
    let hist_len = history.len();
    for order in (0..=hist_len).rev() {
        match traverse_context(&mut model.context, &history[hist_len - order..hist_len]) {
            None => { },
            Some(ctx) => {
                if symbol != EOF && ctx.frequencies.get(symbol) > 0 {
                    return encoder.write(&mut ctx.frequencies, symbol);
                }
                // Else write context escape symbol and continue decrementing the order
                encoder.write(&mut ctx.frequencies, EOF)?;
            },
        }
    }
    // Logic for order = -1
    encoder.write(&mut model.order_minus1_freqs, symbol)
}

/// Try to use highest order context that exists based on the history suffix. When symbol 256
/// is consumed at a context at any non-negative order, it means "escape to the next lower order
/// with non-empty context". When symbol 256 is consumed at the order -1 context, it means "EOF".
fn decode_symbol<'a, R: Read>(model: &'a mut PPMModel, history: &[Symbol], decoder: &mut ArithmeticDecoder<R>) -> Result<Symbol> {
    let hist_len = history.len();
    for order in (0..=hist_len).rev() {
        match traverse_context(&mut model.context, &history[hist_len - order..hist_len]) {
            None => { },
            Some(ref mut ctx) => {
                let symbol = decoder.read(&mut ctx.frequencies)?;
                if symbol < EOF {
                    return Ok(symbol);
                }
                // Else we read the context escape symbol, so continue decrementing the order
            },
        }
    }
    // Logic for order = -1
    decoder.read(&mut model.order_minus1_freqs)
}

#[cfg(test)]
mod tests {
    use std::io::Result;
    use test::Bencher;

    use super::{apply, reduce};

    #[test]
    fn test_compression() -> Result<()> {
        let original: Vec<u8> = String::from("\
            Lorem Ipsum is simply dummy text of the printing and typesetting industry.\
            Lorem Ipsum has been the industry's standard dummy text ever since the 1500s,\
            when an unknown printer took a galley of type and scrambled it to make a type \
            specimen book. It has survived not only five centuries, but also the leap into \
            electronic typesetting, remaining essentially unchanged. It was popularised in \
            the 1960s with the release of Letraset sheets containing Lorem Ipsum passages, \
            and more recently with desktop publishing software like Aldus PageMaker including \
            versions of Lorem Ipsum.").into_bytes();

        let intermediate = apply(&original)?;
        let restored = reduce(&intermediate)?;

        assert_eq!(original, restored);
        Ok(())
    }

    #[bench]
    fn bench_compression(b: &mut Bencher) {
        let original: Vec<u8> = String::from("\
            Lorem Ipsum is simply dummy text of the printing and typesetting industry.\
            Lorem Ipsum has been the industry's standard dummy text ever since the 1500s,\
            when an unknown printer took a galley of type and scrambled it to make a type \
            specimen book. It has survived not only five centuries, but also the leap into \
            electronic typesetting, remaining essentially unchanged. It was popularised in \
            the 1960s with the release of Letraset sheets containing Lorem Ipsum passages, \
            and more recently with desktop publishing software like Aldus PageMaker including \
            versions of Lorem Ipsum.").into_bytes();
        b.iter(|| {
            let intermediate = apply(&original).unwrap();
            reduce(&intermediate).unwrap();
        });
    }
}