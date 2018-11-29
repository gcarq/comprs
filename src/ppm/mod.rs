use bitbit::{BitReader, BitWriter};
use coding::{FrequencyTable, Symbol};
use coding::decoder::ArithmeticDecoder;
use coding::encoder::ArithmeticEncoder;
use ppm::context::Context;
use ppm::model::PPMModel;
use std::io::{Read, Result, Write};

pub mod context;
pub mod model;

const EOF: Symbol = 256;

/// Compress content provided by reader and write compressed data to writer.
pub fn compress<R: Read, W: Write>(reader: &mut R, writer: &mut W, order: isize, symbol_limit: u16, escape_symbol: Symbol) -> Result<()> {
    if order == -1 {
        debug!("DEBUG: order 0 requested. bypassing compression");
        let mut buffer = [0u8; 4096];
        while reader.read(&mut buffer)? > 0 {
            writer.write_all(&buffer)?;
        }
        return writer.flush();
    }

    let mut encoder = ArithmeticEncoder::new(BitWriter::new(writer), 32);
    let mut model = PPMModel::new(order as u8, symbol_limit, escape_symbol);
    let mut history: Vec<u16> = Vec::with_capacity(model.order as usize + 1);


    let mut buffer = [0u8; 1];
    // Read bytes to buffer from given reader
    while reader.read(&mut buffer)? > 0 {
        let symbol = Symbol::from(buffer[0]);
        encode_symbol(&mut model, &history, symbol, &mut encoder)?;
        model.increment_contexts(&history, symbol);

        if model.order >= 1 {
            mutate_history(symbol, model.order, &mut history);
        }
    }

    // Encode EOF
    encode_symbol(&mut model, &history, 256, &mut encoder)?;
    encoder.finish()
}

/// Decompress content provided by reader and write restored data to writer.
pub fn decompress<R: Read, W: Write>(reader: &mut R, writer: &mut W, order: isize, symbol_limit: u16, escape_symbol: Symbol) -> Result<()> {

    if order == -1 {
        debug!("DEBUG: order 0 requested. bypassing compression");
        let mut buffer = [0u8; 4096];
        while reader.read(&mut buffer)? > 0 {
            writer.write_all(&buffer)?;
        }
        return writer.flush();
    }

    let mut decoder = ArithmeticDecoder::new(BitReader::new(reader), 32)?;
    let mut model = PPMModel::new(order as u8, symbol_limit, escape_symbol);
    let mut history: Vec<u16> = Vec::with_capacity(model.order as usize + 1);

    loop {
        let symbol = decode_symbol(&mut model, &history, &mut decoder)?;
        // Check if EOF symbol has occurred
        if symbol == EOF {
            break
        }
        writer.write_all(&[symbol as u8])?;
        model.increment_contexts(&history, symbol);

        if model.order >= 1 {
            mutate_history(symbol, model.order, &mut history);
        }
    }
    writer.flush()
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
                if symbol < 256 {
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
    use std::io::{Cursor, Result, Seek, SeekFrom};
    use super::{compress, decompress};

    #[test]
    fn test_compression() -> Result<()> {
        let test_data: Vec<u8> = String::from("\
            Lorem Ipsum is simply dummy text of the printing and typesetting industry.\
            Lorem Ipsum has been the industry's standard dummy text ever since the 1500s,\
            when an unknown printer took a galley of type and scrambled it to make a type \
            specimen book. It has survived not only five centuries, but also the leap into \
            electronic typesetting, remaining essentially unchanged. It was popularised in \
            the 1960s with the release of Letraset sheets containing Lorem Ipsum passages, \
            and more recently with desktop publishing software like Aldus PageMaker including \
            versions of Lorem Ipsum.").into_bytes();

        let mut original = Cursor::new(test_data);
        let mut intermediate = Cursor::new(Vec::new());
        let mut restored = Cursor::new(Vec::new());

        compress(&mut original, &mut intermediate, 3, 257, 256)?;
        intermediate.seek(SeekFrom::Start(0))?;

        decompress(&mut intermediate, &mut restored, 3, 257, 256)?;
        restored.seek(SeekFrom::Start(0))?;

        assert_eq!(original.into_inner(), restored.into_inner());
        Ok(())
    }
}