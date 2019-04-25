use std::io::{Cursor, Read, Result, Seek, Write};

use varuint::{Deserializable, Serializable, Varint};

pub fn apply(data: &[u8]) -> Result<Vec<u8>> {
    let len = data.len();
    debug!("DEBUG:RLE: before: {}", len);
    let mut writer = Cursor::new(Vec::with_capacity(data.len()));
    let mut pair = RLEPair::new(1, data[0]);

    for symbol in &data[1..] {
        if pair.symbol == *symbol {
            // Increment counter if symbol is the same
            pair.increment();
        } else {
            // Serialize current pair and create new one
            pair.serialize(&mut writer)?;
            pair = RLEPair::new(1, *symbol);
        }
    }
    pair.serialize(&mut writer)?;

    let encoded = writer.into_inner();
    debug!("DEBUG:RLE: after: {}", encoded.len());
    Ok(encoded)
}

pub fn reduce(data: &[u8]) -> Vec<u8> {
    let mut decoded: Vec<u8> = Vec::with_capacity(data.len());

    let mut reader = Cursor::new(data);
    while let Ok(pair) = RLEPair::deserialize(&mut reader) {
        for _ in 0..pair.count.0 {
            decoded.push(pair.symbol);
        }
    }

    decoded
}

struct RLEPair {
    pub count: Varint<u32>,
    pub symbol: u8,
}

impl RLEPair {
    pub fn new(count: usize, symbol: u8) -> Self {
        RLEPair {count: Varint(count as u32), symbol}
    }

    pub fn increment(&mut self) {
        self.count.0 += 1;
    }

    pub fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.count.serialize(writer).expect("unable to serialize varuint");
        writer.write_all(&[self.symbol])
    }

    pub fn deserialize<R: Read+Seek>(reader: &mut R) -> Result<Self> {
        let count = Varint::deserialize(reader)?;

        let mut buf = [0u8; 1];
        reader.read_exact(&mut buf)?;
        let symbol = buf[0];

        Ok(RLEPair {count, symbol})
    }
}

#[cfg(test)]
mod tests {
    use super::{apply, reduce};

    #[test]
    fn test_apply() {
        let data = String::from("WWWWWWWWWWWWBWWWWWWWWWWWWBBBWWWWWWWWWWWWWWWWWWWWWWWWBWWWWWWWWWWWWWW").into_bytes();

        let test_data = [12, 87, 1, 66, 12, 87, 3, 66, 24, 87, 1, 66, 14, 87];
        assert_eq!(apply(&data).unwrap(), test_data);
    }

    #[test]
    fn test_reduce() {
        let data = [12, 87, 1, 66, 12, 87, 3, 66, 24, 87, 1, 66, 14, 87];
        assert_eq!(reduce(&data), String::from("WWWWWWWWWWWWBWWWWWWWWWWWWBBBWWWWWWWWWWWWWWWWWWWWWWWWBWWWWWWWWWWWWWW").into_bytes());
    }
}