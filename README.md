# comprs

[![Build Status](https://travis-ci.org/gcarq/comprs.svg?branch=master)](https://travis-ci.org/gcarq/comprs) [![Coverage Status](https://coveralls.io/repos/github/gcarq/comprs/badge.svg)](https://coveralls.io/github/gcarq/comprs)

Experimental playground for compression algorithms in Rust.
PPM and Arithmetic coder are inspired by: [Reference-arithmetic-coding](https://github.com/nayuki/Reference-arithmetic-coding).

Currently implemented algorithms:
* [Prediction by Partial Matching](https://en.wikipedia.org/wiki/Prediction_by_partial_matching)
* [Arithmetic coding](https://en.wikipedia.org/wiki/Arithmetic_coding)
* [Burrows-Wheeler transform](https://en.wikipedia.org/wiki/Burrows%E2%80%93Wheeler_transform)
* [Move-to-front transform](https://en.wikipedia.org/wiki/Move-to-front_transform)
* [Run-length encoding](https://en.wikipedia.org/wiki/Run-length_encoding)

## Usage

```
 comprs 0.1.0
Experimental playground for compression algorithms in Rust

USAGE:
    comprs [FLAGS] [OPTIONS] <mode> <file>

FLAGS:
    -h, --help       Prints help information
    -n               Skip integrity check
    -v               Sets the level of verbosity
    -V, --version    Prints version information

OPTIONS:
    -o <o>        Specify compression level [default: 3]  [possible values: 0, 1, 2, 3, 4, 5, 6]

ARGS:
    <mode>    mode [possible values: c, d, compress, decompress]
    <file>    Sets the input file to use
```

## Building

```
$ git clone https://github.com/gcarq/comprs.git
$ cd comprs
$ cargo build --release
```

## Example

```
$ wget -O world95.txt https://www.gutenberg.org/files/27560/27560.txt
$ ./target/release/comprs c world95.txt
Applying preprocessors ...
 -> BWT
 -> MTF
Compressing file ...
Compressed Size: 1683194
Compress Ratio: 5.1 (80.57%)
Bits per Byte: 1.5542
Verifying compressed file ...
Decoding preprocessors ...
 -> MTF
 -> BWT
checksum is OK - 1921997141
```
