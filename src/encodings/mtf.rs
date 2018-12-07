

pub fn apply(data: &[u8]) -> Vec<u8> {
    println!(" -> MTF");
    // Create alphabet vector
    let mut alphabet: Vec<u8> = (0..=255).collect();

    // Iterates over data and encodes each byte with the current alphabet
    data.into_iter().map(|b|encode(*b, &mut alphabet)).collect()
}

pub fn reduce(data: &[u8]) -> Vec<u8> {
    println!(" -> MTF");

    // Create alphabet vector
    let mut alphabet: Vec<u8> = (0..=255).collect();

    let mut decoded = Vec::with_capacity(data.len());
    for index in data {
        let byte = alphabet.remove(*index as usize);
        decoded.push(byte);
        alphabet.insert(0, byte);
    }

    decoded
}

fn encode(byte: u8, alphabet: &mut Vec<u8>) -> u8 {
    let index = alphabet.iter()
        .position(|&b| b == byte)
        .expect("byte not found in alphabet");

    let byte = alphabet.remove(index);
    alphabet.insert(0, byte);
    index as u8
}

#[cfg(test)]
mod tests {
    use super::{apply, reduce};

    #[test]
    fn test_apply() {
        let data = String::from("bananaaa").into_bytes();
        assert_eq!(apply(&data), vec![98, 98, 110, 1, 1, 1, 0, 0]);
    }

    #[test]
    fn test_reduce() {
        let data = vec![98, 98, 110, 1, 1, 1, 0, 0];
        assert_eq!(reduce(&data), String::from("bananaaa").into_bytes());
    }
}