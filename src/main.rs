#![feature(test)]

extern crate bincode;
extern crate bitbit;
#[macro_use] extern crate clap;
#[macro_use] extern crate log;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate test;
extern crate varuint;
extern crate adler32;
extern crate rayon;


use clap::{App, Arg};
use adler32::adler32;
use std::fs::File;
use std::io::{BufReader, BufWriter, Cursor, Read, Result, Write};
use utils::print_statistics;

mod utils;
mod encodings;


fn main() -> Result<()> {
    let matches = App::new("comprs")
        .version(crate_version!())
        .about("Experimental sandbox for compression algorithms in Rust")
        .arg(Arg::with_name("mode")
            .help("mode")
            .required(true)
            .possible_values(&["c", "d", "compress", "decompress"])
            .index(1))
        .arg(Arg::with_name("file")
            .help("Sets the input file to use")
            .required(true)
            .index(2))
        .arg(Arg::with_name("o")
            .short("o")
            .takes_value(true)
            .default_value("3")
            .possible_values(&["0", "1", "2", "3", "4", "5", "6"])
            .help("Specify compression level"))
        .arg(Arg::with_name("v")
            .short("v")
             .multiple(true)
             .help("Sets the level of verbosity"))
        .arg(Arg::with_name("no-verify")
            .short("n")
            .multiple(false)
            .help("Skip integrity check"))
        .get_matches();

    let input_file = String::from(matches.value_of("file").unwrap());
    let mut verify = true;
    if matches.is_present("no-verify") {
        verify = false;
    };

    match matches.value_of("mode").unwrap() {
        "c" | "compress" => {
            let mut reader = BufReader::new(File::open(&input_file)?);
            let output_file = format!("{}.comprs", input_file.clone());
            debug!("DEBUG: Saving output to: {}", &output_file);
            let mut writer = BufWriter::new(File::create(&output_file)?);

            writer.write_all(&compress_file(&mut reader)?)?;

            print_statistics(
                &File::open(&input_file)?.metadata()?,
                &File::open(&output_file)?.metadata()?);

            if !verify {
                return Ok(());
            }

            println!("Verifying compressed file ...");

            let mut restored = decompress_file(&mut BufReader::new(File::open(&output_file)?))?;

            // Calculate checksums
            let input_checksum = adler32(&mut File::open(&input_file)?)?;
            let restored_checksum = adler32(restored.as_slice())?;

            // Sanity check
            if input_checksum == restored_checksum {
                println!("checksum is OK - {}", restored_checksum);
            } else {
                panic!(format!("FATAL: checksum does not match! - {}", restored_checksum));
            }
        },
        "d" | "decompress" => {
            let output_file = input_file.clone().replace(".comprs", ".restored");
            let mut reader = BufReader::new(File::open(input_file)?);
            let mut writer = BufWriter::new(File::create(&output_file)?);
            writer.write_all(&decompress_file(&mut reader)?)?;
        },
        _ => unreachable!(),
    }
    Ok(())
}

fn compress_file<R: Read>(reader: R) -> Result<Vec<u8>> {
    println!("Compressing file ...");
    let cursor = Cursor::new(encodings::encode_pipeline(reader)?);
    Ok(cursor.into_inner())
}


fn decompress_file<R: Read>(reader: R) -> Result<Vec<u8>> {
    println!("Reverting encodings ...");
    encodings::decode_pipeline(reader)
}


#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[test]
    fn test_compression() -> Result<()> {
        let test_data = String::from("Lorem Ipsum is simply dummy text of the printing and typesetting industry.
            Lorem Ipsum has been the industry's standard dummy text ever since the 1500s,
            when an unknown printer took a galley of type and scrambled it to make a type specimen book.
            It has survived not only five centuries, but also the leap into electronic typesetting,
            remaining essentially unchanged. It was popularised in the 1960s with the release of
            Letraset sheets containing Lorem Ipsum passages, and more recently with desktop publishing
            software like Aldus PageMaker including versions of Lorem Ipsum.
            It is a long established fact that a reader will be distracted by the readable content
            of a page when looking at its layout. The point of using Lorem Ipsum is that it has a
            more-or-less normal distribution of letters, as opposed to using 'Content here, content here',
            making it look like readable English. Many desktop publishing packages and web page editors now
            use Lorem Ipsum as their default model text, and a search for 'lorem ipsum' will uncover many
            web sites still in their infancy. Various versions have evolved over the years,
            sometimes by accident, sometimes on purpose (injected humour and the like).
            It is a long established fact that a reader will be distracted by the
            readable content of a page when looking at its layout.
            The point of using Lorem Ipsum is that it has a more-or-less normal
            distribution of letters, as opposed to using 'Content here, content here',
            making it look like readable English. Many desktop publishing packages and
            web page editors now use Lorem Ipsum as their default model text, and a search for
            'lorem ipsum' will uncover many web sites still in their infancy. Various versions
            have evolved over the years, sometimes by accident, sometimes on purpose
            (injected humour and the like). There are many variations of passages of
            Lorem Ipsum available, but the majority have suffered alteration in some form,
            by injected humour, or randomised words which don't look even slightly believable.
            If you are going to use a passage of Lorem Ipsum, you need to be sure there isn't
            anything embarrassing hidden in the middle of text. All the Lorem Ipsum generators
            on the Internet tend to repeat predefined chunks as necessary,
            making this the first true generator on the Internet.
            It uses a dictionary of over 200 Latin words,
            combined with a handful of model sentence structures,
            to generate Lorem Ipsum which looks reasonable. The generated Lorem Ipsum is
            therefore always free from repetition, injected humour, or non-characteristic words etc."
        ).into_bytes();

        let compressed = compress_file(test_data.as_slice())?;
        let restored = decompress_file(compressed.as_slice())?;

        assert_eq!(restored, test_data);
        Ok(())
    }

    #[bench]
    fn bench_compress_file(b: &mut Bencher) {
        let test_data = "Lorem Ipsum is simply dummy text of the printing and typesetting industry.
            Lorem Ipsum has been the industry's standard dummy text ever since the 1500s,
            when an unknown printer took a galley of type and scrambled it to make a type specimen book.
            It has survived not only five centuries, but also the leap into electronic typesetting,
            remaining essentially unchanged. It was popularised in the 1960s with the release of
            Letraset sheets containing Lorem Ipsum passages, and more recently with desktop publishing
            software like Aldus PageMaker including versions of Lorem Ipsum.
            It is a long established fact that a reader will be distracted by the readable content
            of a page when looking at its layout. The point of using Lorem Ipsum is that it has a
            more-or-less normal distribution of letters, as opposed to using 'Content here, content here',
            making it look like readable English. Many desktop publishing packages and web page editors now
            use Lorem Ipsum as their default model text, and a search for 'lorem ipsum' will uncover many
            web sites still in their infancy. Various versions have evolved over the years,
            sometimes by accident, sometimes on purpose (injected humour and the like).
            It is a long established fact that a reader will be distracted by the
            readable content of a page when looking at its layout.
            The point of using Lorem Ipsum is that it has a more-or-less normal
            distribution of letters, as opposed to using 'Content here, content here',
            making it look like readable English. Many desktop publishing packages and
            web page editors now use Lorem Ipsum as their default model text, and a search for
            'lorem ipsum' will uncover many web sites still in their infancy. Various versions
            have evolved over the years, sometimes by accident, sometimes on purpose
            (injected humour and the like). There are many variations of passages of
            Lorem Ipsum available, but the majority have suffered alteration in some form,
            by injected humour, or randomised words which don't look even slightly believable.
            If you are going to use a passage of Lorem Ipsum, you need to be sure there isn't
            anything embarrassing hidden in the middle of text. All the Lorem Ipsum generators
            on the Internet tend to repeat predefined chunks as necessary,
            making this the first true generator on the Internet.
            It uses a dictionary of over 200 Latin words,
            combined with a handful of model sentence structures,
            to generate Lorem Ipsum which looks reasonable. The generated Lorem Ipsum is
            therefore always free from repetition, injected humour, or non-characteristic words etc.";
        b.iter(|| {
            let mut reader = BufReader::new(Cursor::new(String::from(test_data).into_bytes()));
            let mut buffer = Vec::new();

            buffer.write_all(&compress_file(&mut reader).unwrap()).unwrap();
        });
    }
}


