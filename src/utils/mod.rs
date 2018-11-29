use sha2::{Digest, Sha256};
use std::fs::Metadata;
use std::io::{Error, Read};

const BUFFER_SIZE: usize = 1024;

/// Compute digest value for given `Reader` and print it.
pub fn sha256sum<R: Read>(reader: &mut R) -> Result<String, Error> {
    let mut sh = Sha256::default();
    let mut buffer = [0u8; BUFFER_SIZE];
    loop {
        let n = reader.read(&mut buffer)?;
        sh.input(&buffer[..n]);
        if n == 0 || n < BUFFER_SIZE {
            break;
        }
    }

    Ok(
        sh.result().iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<String>>()
        .join("")
    )
}

pub fn calc_entropy(data: &[u8]) -> f64 {
    // Calculate frequency of each byte in vector
    let flen = data.len() as f64;
    let mut freqlist = Vec::with_capacity(256);
    for symbol in 0..255 {
        let mut ctr = 0;
        for byte in data {
            if *byte == symbol {
                ctr += 1;
            }
        }
        freqlist.push(f64::from(ctr) / flen);
    }

    let mut entropy = 0.0;
    // Calculate shannon entropy
    for freq in freqlist {
        if freq > 0.0 {
            entropy += freq * freq.log2();
        }
    }
    -entropy
}

pub fn print_statistics(input_meta: &Metadata, compressed_meta: &Metadata) {
    let input_size = input_meta.len() as f64;
    let comp_size = compressed_meta.len() as f64;
    println!("Compressed Size: {}", comp_size);
    println!("Compress Ratio: {:.1} ({:.2}%)",
             input_size / comp_size,
             (1.0 - comp_size / input_size) * 100.0);
    println!("Bits per Byte: {:.4}", comp_size / input_size * 8.0);
}