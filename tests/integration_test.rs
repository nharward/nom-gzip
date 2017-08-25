extern crate nom;
extern crate nom_gzip;

use nom::IResult::Done;

use nom_gzip::*;
use nom_gzip::types::*;

const SAMPLE_GZIP_FILE: &'static [u8] = include_bytes!("sample.txt.gz");
const HEADER_SIZE: usize = 10 + (10 + 1); // 10 bytes fixed + (original filename + null terminator)
const FOOTER_SIZE: usize = 8;

fn validate_header(header: &GzipHeader) {
    assert_eq!(header.compression_method, CompressionMethod::Deflate);
    assert!(! header.flags.ftext);
    assert!(! header.flags.fhcrc);
    assert!(! header.flags.fextra);
    assert!(header.flags.fname);
    assert!(! header.flags.fcomment);
    assert_eq!(header.modified_time_as_secs_since_epoch.as_secs(), 0x599e86e7);
    assert_eq!(header.extra_flags, ExtraFlags::MaximumCompression);
    assert_eq!(header.operating_system, OperatingSystem::Unix);
    assert_eq!(header.extra_field, None);
    assert_eq!(header.original_filename, Some(String::from("sample.txt")));
    assert_eq!(header.file_comment, None);
    assert_eq!(header.header_crc, None);
}

fn validate_footer(footer: &GzipFooter) {
    assert_eq!(footer.crc, 0xbd47c3dc);
    assert_eq!(footer.input_size, 0x0000738f);
}

#[test]
fn it_header() {
    match gzip_header(SAMPLE_GZIP_FILE) {
        Done(remaining, header) => {
            validate_header(&header);
            assert_eq!(remaining.len(), SAMPLE_GZIP_FILE.len() - HEADER_SIZE);
        }
        unexpected => assert!(false, "Expected a GZIP header, got this instead: {:?}", unexpected),
    }
}

#[test]
fn it_footer() {
    match gzip_footer(&SAMPLE_GZIP_FILE[(SAMPLE_GZIP_FILE.len() - FOOTER_SIZE)..]) {
        Done(remaining, footer) => {
            validate_footer(&footer);
            assert_eq!(remaining.len(), 0);
        },
        unexpected => assert!(false, "Expected a GZIP footer, got this instead: {:?}", unexpected),
    }
}

#[test]
fn it_whole_file() {
    match gzip_file(SAMPLE_GZIP_FILE) {
        Done(_, gz_file) => {
            validate_header(&gz_file.header);
            assert_eq!(gz_file.compressed_blocks.len(), SAMPLE_GZIP_FILE.len() - HEADER_SIZE - FOOTER_SIZE);
            validate_footer(&gz_file.footer);
        },
        unexpected => assert!(false, "Expected a GZIP file, got this instead: {:?}", unexpected),
    }
}
