use bincode;
use std::io::{BufReader, Read, Result};
use utils::calc_entropy;

mod bwt;
mod mtf;
mod rle;
mod startransform;
mod custom;


#[derive(Serialize, Deserialize)]
pub enum Transform {
    BWT,
    MTF,
    RLE,
    ST,
    CUSTOM,
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
            //Transform::CUSTOM,
            //Transform::ST, FIXME: currently broken
            Transform::BWT,
            Transform::MTF,
        ];

        for transform in &transforms {
            buffer = match transform {
                Transform::ST => startransform::apply(&buffer),
                Transform::BWT => bwt::apply(&buffer),
                Transform::MTF => mtf::apply(&buffer),
                Transform::RLE => rle::apply(&buffer)?,
                Transform::CUSTOM => custom::apply(&buffer),
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
            buffer = match transform {
                Transform::BWT => bwt::reduce(&buffer),
                Transform::MTF => mtf::reduce(&buffer),
                Transform::RLE => rle::reduce(&buffer),
                Transform::CUSTOM => custom::reduce(&buffer),
                _ => unimplemented!("notimplemented"),
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
    let mut buffer = Vec::new();
    BufReader::new(reader).read_to_end(&mut buffer)?;

    let data = bincode::deserialize::<TData>(&buffer)
        .expect("unable to deserialize data");
    data.decode()
}