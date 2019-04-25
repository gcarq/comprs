use std::fmt;
use std::io::{BufReader, Read, Result};

use bincode;

use utils::calc_entropy;

pub mod ppm;
pub mod bwt;
pub mod mtf;
pub mod rle;
pub mod startransform;
pub mod arithmetic_coder;

#[derive(Serialize, Deserialize)]
pub enum Transform {
    BWT,
    MTF,
    RLE,
    ST,
    PPM,
}

impl fmt::Display for Transform {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match *self {
            Transform::BWT => "BWT",
            Transform::MTF => "MTF",
            Transform::RLE => "RLE",
            Transform::ST => "ST",
            Transform::PPM => "PPM",
        };
        write!(f, "{}", printable)
    }
}

#[derive(Serialize, Deserialize)]
pub struct TData {
    pub transforms: Vec<Transform>,
    pub buffer: Vec<u8>,
}


impl TData {
    /// Applies given transform methods to reader input
    pub fn encode<R: Read>(reader: R) -> Result<TData> {
        let mut buffer = Vec::with_capacity(1024 * 1024);

        BufReader::new(reader).read_to_end(&mut buffer)?;

        debug!("DEBUG: Size before preprocessing: {}", &buffer.len());
        debug!("DEBUG: File entropy before preprocessing: {:.2}", calc_entropy(&buffer));

        let transforms = vec![
            //Transform::ST, FIXME: currently broken
            Transform::BWT,
            Transform::MTF,
            Transform::PPM,
        ];

        for transform in &transforms {
            println!("  -> {} ", transform);
            buffer = match transform {
                Transform::ST => startransform::apply(&buffer),
                Transform::BWT => bwt::apply(&buffer),
                Transform::MTF => mtf::apply(&buffer),
                Transform::RLE => rle::apply(&buffer)?,
                Transform::PPM => ppm::apply(&buffer)?,
            };
        }

        debug!("DEBUG: Size after preprocessing: {}", &buffer.len());
        debug!("DEBUG: File entropy after preprocessing: {:.2}", calc_entropy(&buffer));
        Ok(TData {transforms, buffer})
    }

    /// Decodes self  and returns the content as bytes
    pub fn decode(self) -> Result<Vec<u8>> {
        let mut buffer = self.buffer;

        for transform in self.transforms.iter().rev() {
            println!("  -> {} ", transform);
            buffer = match transform {
                Transform::BWT => bwt::reduce(&buffer),
                Transform::MTF => mtf::reduce(&buffer),
                Transform::RLE => rle::reduce(&buffer),
                Transform::PPM => ppm::reduce(&buffer)?,
                _ => unimplemented!("not implemented"),
            };
        }
        Ok(buffer)
    }
}

pub fn encode_pipeline<R: Read>(reader: R) -> Result<Vec<u8>> {
    Ok(bincode::serialize(&TData::encode(reader)?)
        .expect("unable to serialize data"))
}

pub fn decode_pipeline<R: Read>(reader: R) -> Result<Vec<u8>> {
    let data = bincode::deserialize_from::<R, TData>(reader)
        .expect("unable to deserialize data");
    data.decode()
}
