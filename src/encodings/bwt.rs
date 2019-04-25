use std::cmp::Ordering;
use std::ops::Index;

use bincode;
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
use rayon::prelude::ParallelSliceMut;

const CHUNK_SIZE: u32 = 1024 * 1024;


pub fn apply(data: &[u8]) -> Vec<u8> {
    // Create chunks and encode them
    let chunks: Vec<BWTChunk> = data.chunks(CHUNK_SIZE as usize)
        .map(BWTChunk::encode)
        .collect();
    debug!("DEBUG:BWT: split up into {} chunks", chunks.len());

    // Serialize encoded data to u8
    bincode::serialize(&BWTData {chunks})
        .expect("unable to serialize data")
}

pub fn reduce(data: &[u8]) -> Vec<u8> {
    // Create chunks and encode them
    let data: BWTData = bincode::deserialize(data)
        .expect("unable to deserialize data");
    debug!("DEBUG:BWT: got {} chunks", data.chunks.len());

    data.chunks.into_iter()
        .map(BWTChunk::decode)
        .flatten()
        .collect()
}

struct BWTReconstructData {
    pub position: u32,
    pub char: u8,
}

impl Ord for BWTReconstructData {
    fn cmp(&self, other: &BWTReconstructData) -> Ordering {
        self.char.cmp(&other.char)
    }
}

impl PartialOrd for BWTReconstructData {
    fn partial_cmp(&self, other: &BWTReconstructData) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for BWTReconstructData {
    fn eq(&self, other: &BWTReconstructData) -> bool {
        self.char == other.char
    }
}

impl Eq for BWTReconstructData { }

#[derive(Serialize, Deserialize)]
struct BWTData {
    pub chunks: Vec<BWTChunk>,
}

#[derive(Serialize, Deserialize)]
struct BWTChunk {
    pub data: Vec<u8>,
    pub index: u32,
}

impl BWTChunk {
    pub fn encode(data: &[u8]) -> Self {
        let len = data.len();

        // Create permutations table
        let mut permutations: Vec<Permutation> = (0..len)
            .into_par_iter()
            .map(|i| Permutation::new(data, i as u32)).collect();

        permutations.par_sort();

        // Create encoded data by using the last element in each row
        let index: u32 = permutations.par_iter()
            .position_any(|p| p.index == 0).unwrap() as u32;
        let data: Vec<u8> = permutations.par_iter()
            .map(|m| m[len-1]).collect();

        BWTChunk {data, index}
    }

    fn decode(self) -> Vec<u8> {
        let len = self.data.len();

        // Save all characters with along with position
        let mut table: Vec<BWTReconstructData> = self.data.par_iter().enumerate()
            .map(|(i, c)| BWTReconstructData {position: i as u32, char: *c})
            .collect();

        table.par_sort();

        // Build decoded content
        let mut decoded = Vec::with_capacity(len);
        let mut idx: usize = self.index as usize;
        for _ in 0..len {
            decoded.push(table[idx].char);
            idx = table[idx].position as usize;
        }
        decoded
    }
}

struct Permutation<'a> {
    data: &'a [u8],
    pub index: u32,
}

impl<'a> Permutation<'a> {
    pub fn new(data: &'a [u8], index: u32) -> Self {
        Permutation { data, index }
    }
}

impl<'a> Index<usize> for Permutation<'a> {
    type Output = u8;

    /* 01234567
    0: .ANANAS.
    1: ..ANANAS
    2: S..ANANA
    3: AS..ANAN
    4: NAS..ANA
    5: ANAS..AN
    6: NANAS..A
    7: ANANAS..
    */
    fn index(&self, idx: usize) -> &u8 {
        let len = self.data.len();
        &self.data[(len - self.index as usize + idx) % len]
    }
}

impl<'a> Ord for Permutation<'a> {
    fn cmp(&self, other: &Permutation<'a>) -> Ordering {
        if self.index != other.index {
            for i in 0..self.data.len() {
                match self[i].cmp(&other[i]) {
                    Ordering::Equal => continue,
                    o => return o
                }
            }
        }
        Ordering::Equal
    }
}

impl<'a> PartialOrd for Permutation<'a> {
    fn partial_cmp(&self, other: &Permutation<'a>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> PartialEq for Permutation<'a> {
    fn eq(&self, other: &Permutation<'a>) -> bool {
        self.index == other.index
    }
}

impl<'a> Eq for Permutation<'a> { }


#[cfg(test)]
mod tests {
    use super::{apply, BWTChunk, reduce};

    #[test]
    fn test_apply() {
        let test_data = vec![
            01, 00, 00, 00, 00, 00, 00, 00,
            19, 00, 00, 00, 00, 00, 00, 00,
            83, 83, 51, 46, 46, 49, 50, 46,
            46, 78, 78, 78, 78, 65, 65, 65,
            65, 65, 65, 02, 00, 00, 00
        ];

        let data = String::from(".ANANAS..ANANAS.123").into_bytes();
        assert_eq!(apply(&data), test_data);
    }

    #[test]
    fn test_reduce() {
        let test_data = vec![
            01, 00, 00, 00, 00, 00, 00, 00,
            19, 00, 00, 00, 00, 00, 00, 00,
            83, 83, 51, 46, 46, 49, 50, 46,
            46, 78, 78, 78, 78, 65, 65, 65,
            65, 65, 65, 02, 00, 00, 00
        ];

        let result = reduce(&test_data);
        assert_eq!(result, String::from(".ANANAS..ANANAS.123").into_bytes());
    }

    #[test]
    fn test_encode() {
        let input: Vec<u8>  = String::from(".ANANAS.").into_bytes();
        let result = BWTChunk::encode(&input);
        assert_eq!(result.index, 1);
        assert_eq!(String::from("S..NNAAA"), String::from_utf8(result.data).unwrap());
    }

    #[test]
    fn test_decode() {
        let input: Vec<u8>  = String::from("S..NNAAA").into_bytes();
        let chunk = BWTChunk {data: input, index: 1};
        assert_eq!(String::from(".ANANAS."), String::from_utf8(chunk.decode()).unwrap());
    }
}