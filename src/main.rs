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


use clap::{App, Arg};
use adler32::adler32;
use coding::Symbol;
use std::fs::File;
use std::io::{BufReader, BufWriter, Cursor, Read, Result, Seek, SeekFrom, Write};
use utils::print_statistics;

mod coding;
mod ppm;
mod utils;
mod preprocessors;


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
    // TODO: remove -1 hack used for no-compression.
    let order: isize = matches.value_of("o").unwrap().parse::<usize>().unwrap() as isize - 1;
    let symbol_limit: u16 = 257;
    let escape_symbol: Symbol = 256;
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

            compress_file(&mut reader, &mut writer, order, symbol_limit, escape_symbol)?;

            print_statistics(
                &File::open(&input_file)?.metadata()?,
                &File::open(&output_file)?.metadata()?);

            if !verify {
                return Ok(());
            }

            println!("Verifying compressed file ...");

            let mut verify_buffer = Cursor::new(Vec::new());
            decompress_file(&mut BufReader::new(File::open(&output_file)?),
                            &mut verify_buffer,
                            order, symbol_limit, escape_symbol)?;

            // Calculate checksums
            let input_checksum = adler32(&mut File::open(&input_file)?)?;
            verify_buffer.seek(SeekFrom::Start(0))?;
            let restored_checksum = adler32(verify_buffer)?;

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
            decompress_file(&mut reader, &mut writer, order, symbol_limit, escape_symbol)?;
        },
        _ => unreachable!(),
    }
    Ok(())
}

fn compress_file<R: Read, W: Write>(reader: &mut R, writer: &mut W, order: isize, symbol_limit: u16, escape_symbol: Symbol) -> Result<()> {
    println!("Applying preprocessors ...");
    let mut cursor = Cursor::new(preprocessors::encode_pipeline(reader)?);

    println!("Compressing file ...");
    ppm::compress(&mut cursor, writer, order, symbol_limit, escape_symbol)?;
    writer.flush()
}


fn decompress_file<R: Read, W: Write>(reader: &mut R, writer: &mut W, order: isize, symbol_limit: u16, escape_symbol: Symbol) -> Result<()> {
    // Create in-memory writer to be used to reduce pre-processors afterwards
    let mut cursor = Cursor::new(Vec::new());
    ppm::decompress(reader, &mut cursor, order, symbol_limit, escape_symbol)?;
    cursor.seek(SeekFrom::Start(0))?;

    println!("Decoding preprocessors ...");
    let data = preprocessors::decode_pipeline(cursor)?;
    writer.write_all(&data)?;
    writer.flush()
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

        let mut reader = Cursor::new(&test_data);
        let mut cursor = Cursor::new(Vec::new());
        let mut writer = Cursor::new(Vec::new());

        compress_file(&mut reader, &mut cursor, 1, 257, 256)?;
        cursor.seek(SeekFrom::Start(0))?;
        decompress_file(&mut cursor, &mut writer, 1, 257, 256)?;
        let result = writer.into_inner();

        assert_eq!(result, test_data);
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
            let mut writer = BufWriter::new(Cursor::new(Vec::new()));

            compress_file(&mut reader, &mut writer, 1, 257, 256).unwrap();
        });
    }
}


