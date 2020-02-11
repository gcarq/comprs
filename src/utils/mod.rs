use std::collections::HashMap;
use std::fs::Metadata;

/// Calculates shannon entropy for the given slice
pub fn calc_entropy(data: &[u8]) -> f64 {
    let mut occurences = HashMap::new();
    for byte in data {
        *occurences.entry(byte).or_insert(0) += 1;
    }

    let flen = data.len() as f64;
    let entropy: f64 = occurences
        .values()
        .map(|o| f64::from(*o) / flen)
        .map(|p| p * p.log2())
        .sum();

    -entropy
}

pub fn print_statistics(input_meta: &Metadata, compressed_meta: &Metadata) {
    let input_size = input_meta.len() as f64;
    let comp_size = compressed_meta.len() as f64;
    println!("Compressed Size: {}", comp_size);
    println!(
        "Compress Ratio: {:.1} ({:.2}%)",
        input_size / comp_size,
        (1.0 - comp_size / input_size) * 100.0
    );
    println!("Bits per Byte: {:.4}", comp_size / input_size * 8.0);
}

#[cfg(test)]
mod tests {
    use crate::utils::calc_entropy;

    #[test]
    fn test_calc_entropy() {
        let data: Vec<u8> = String::from("Lorem ipsum").into_bytes();
        let result = calc_entropy(&data);
        assert_eq!(format!("{:.5}", result), "3.27761");
    }
}
