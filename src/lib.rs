//! [nom](https://docs.rs/nom/3.2.0/nom/) parser for the GZIP file format, as documented in [RFC
//! 1952](https://tools.ietf.org/rfc/rfc1952.txt).
//!
//! # Notes on this parser
//!
//! ## TL;DR
//!
//! This parser assumes the GZIP data contains only a single compressed file that goes until EOF.
//!
//! ## Details
//!
//! While in theory multiple files can be in a single GZIP stream by simply concatenating multiple
//! GZIP files together (see [section 2.2](https://tools.ietf.org/html/rfc1952#page-5]) of the RFC),
//! in practice it appears that at least the GZIP and 7z utilities (in Linux) do not correctly
//! support this. For two files cat'd together they both report the header of the first file with
//! the footer (uncompressed size of the file) from the second. Decompression of such a file
//! with the gzip utility results in the uncompressed contents of both files concatenated together
//! in a single file instead of two files with separated content. IMHO if this feature of the GZIP
//! format can't be used in any practical sense there is no point in spending time writing a
//! theoretically correct but far more involved (and slower!) parser here.

pub mod types;
use types::*;

#[macro_use]
extern crate nom;

use nom::{le_u16, le_u32};
use nom::Endianness::Little;

use std::time::Duration;

named!(null_terminated_string<String>, map_res!(terminated!(take_until!(&[0x00][..]), take!(1)), |buf: &[u8]| String::from_utf8(buf.to_vec())));
named!(get_byte<u8>, map!(take!(1), |bs| bs[0]));

named!(id1, tag!([0x1f]));
named!(id2, tag!([0x8b]));
named!(compression_method<CompressionMethod>, map!(take!(1), |b| CompressionMethod::from(b[0])));
named!(flags<Flags>, map!(take!(1), |b| Flags::from(b[0])));
named!(modified_time_as_secs_since_epoch<Duration>, map!(u32!(Little), |t| Duration::from_secs(t as u64)));
named!(extra_flags<ExtraFlags>, map!(take!(1), |b| ExtraFlags::from(b[0])));
named!(operating_system<OperatingSystem>, map!(take!(1), |b| OperatingSystem::from(b[0])));

/// What little documentation I could find on existing sub-fields lives at
/// http://www.gzip.org/format.txt but it's woefully inadequate as a spec.
named!(sub_field<SubField>, do_parse!(
       id1: get_byte
    >> id2: get_byte
    >> data: length_data!(le_u16)
    >>
    (SubField { id1, id2, data })
));

named!(extra_field<ExtraField>, length_value!(le_u16, map!(many0!(sub_field), |sub_fields| ExtraField{ sub_fields })));
named!(original_filename<String>, call!(null_terminated_string));
named!(file_comment<String>, call!(null_terminated_string));
named!(header_crc16<u16>, call!(le_u16));
named!(footer_crc32<u32>, call!(le_u32));
named!(input_size<u32>, call!(le_u32));

named!(pub gzip_header<GzipHeader>, do_parse!(
       id1
    >> id2
    >> compression_method: compression_method
    >> flags: flags
    >> modified_time_as_secs_since_epoch: modified_time_as_secs_since_epoch
    >> extra_flags: extra_flags
    >> operating_system: operating_system
    >> extra_field: cond!(flags.fextra, call!(extra_field))
    >> original_filename: cond!(flags.fname, call!(original_filename))
    >> file_comment: cond!(flags.fcomment, call!(file_comment))
    >> header_crc: cond!(flags.fhcrc, call!(header_crc16))
    >>

    (GzipHeader {
        compression_method,
        flags,
        modified_time_as_secs_since_epoch,
        extra_flags,
        operating_system,
        extra_field,
        original_filename,
        file_comment,
        header_crc
    })
));

named!(pub gzip_footer<GzipFooter>, do_parse!(
       crc: footer_crc32
    >> input_size: input_size
    >> eof!()
    >>

    (GzipFooter { crc, input_size })
));

/// This will probably be pretty slow; you'll likely want to use `gzip_header` and then make use of
/// the GZIP stream directly from there, passing in the last 8 bytes to `gzip_footer` if necessary.
named!(pub gzip_file<GzipFile>, do_parse! (
    header: gzip_header
    >> gzip_file: map!(many_till!(call!(get_byte), call!(gzip_footer)), |tup: (Vec<u8>, GzipFooter)| {
                    GzipFile { header, footer: tup.1, compressed_blocks: tup.0.iter().map(|b| *b).collect() }
                  })
    >>

    (gzip_file)
));

#[cfg(test)]
mod tests {

    extern crate nom;
    extern crate byteorder;

    use tests::nom::IResult::Done;

    use super::*;

    macro_rules! empty {
        () => {
            &b""[..];
        }
    }

    macro_rules! test_null_terminated {
        ($func:ident) => {
            let input = &b"This is null-terminated\0"[..];
            let expected = String::from("This is null-terminated");
            match $func(input) {
                Done(_, actual) => assert_eq!(actual, expected),
                unexpected => assert!(false, "Unable to parse null-terminated string, got back {:?}", unexpected),
            }
        }
    }

    macro_rules! test_u16 {
        ($func:ident) => {
            use tests::byteorder::{ByteOrder, LittleEndian};
            let mut buf: [u8; 2] = [0; 2];
            for expected in 0x0000u16 .. 0xffffu16 {
                LittleEndian::write_u16(&mut buf[0..2], expected);
                assert_eq!($func(&buf[..]), Done(empty!(), expected));
            }
        }
    }

    macro_rules! test_u32 {
        ($func:ident) => {
            use tests::byteorder::{ByteOrder, LittleEndian};
            let samples: [u32; 6] = [0x00000000, 0xffffffff, 0xff00ff00, 0x00ff00ff, 0x01234567, 0x89abcdef];
            let mut buf: [u8; 4] = [0; 4];
            for expected in samples.iter() {
                LittleEndian::write_u32(&mut buf[0..4], *expected);
                assert_eq!($func(&buf[..]), Done(empty!(), *expected));
            }
        }
    }

    #[test]
    fn test_id1() {
        let input: &[u8] = &[0x1f][..];
        assert_eq!(id1(input), Done(&b""[..], input));
    }

    #[test]
    fn test_id2() {
        let input: &[u8] = &[0x8b][..];
        assert_eq!(id2(input), Done(&b""[..], input));
    }

    #[test]
    fn test_compression_method() {
        use CompressionMethod::*;
        assert_eq!(compression_method(&[0x00][..]), Done(empty!(), Reserved0));
        assert_eq!(compression_method(&[0x01][..]), Done(empty!(), Reserved1));
        assert_eq!(compression_method(&[0x02][..]), Done(empty!(), Reserved2));
        assert_eq!(compression_method(&[0x03][..]), Done(empty!(), Reserved3));
        assert_eq!(compression_method(&[0x04][..]), Done(empty!(), Reserved4));
        assert_eq!(compression_method(&[0x05][..]), Done(empty!(), Reserved5));
        assert_eq!(compression_method(&[0x06][..]), Done(empty!(), Reserved6));
        assert_eq!(compression_method(&[0x07][..]), Done(empty!(), Reserved7));
        assert_eq!(compression_method(&[0x08][..]), Done(empty!(), Deflate));
        for b in 0x09u8 .. 0xffu8 {
            assert_eq!(compression_method(&[b][..]), Done(empty!(), Unknown));
        }
    }

    #[test]
    fn test_flags() {
        for byte in 0b0000_0000 .. 0b0001_1111 {
            let expected = Done(empty!(), Flags {
                ftext:    byte & 0b0000_0001 > 0,
                fhcrc:    byte & 0b0000_0010 > 0,
                fextra:   byte & 0b0000_0100 > 0,
                fname:    byte & 0b0000_1000 > 0,
                fcomment: byte & 0b0001_0000 > 0,
            });
            assert_eq!(flags(&[byte][..]), expected);
        }
    }

    #[test]
    fn test_modified_time_as_secs_since_epoch() {
        use tests::byteorder::{ByteOrder, LittleEndian};
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now();
        let expected = Duration::from_secs(now.duration_since(UNIX_EPOCH).unwrap().as_secs()); // kill the nanos
        let mut buffer: [u8; 4] = [0; 4];
        LittleEndian::write_u32(&mut buffer[..], expected.as_secs() as u32);
        match modified_time_as_secs_since_epoch(&buffer[..]) {
            Done(remaining, actual) => {
                assert_eq!(remaining, empty!());
                assert_eq!(actual, expected);
            }
            _ => panic!("")
        }
    }

    #[test]
    fn test_extra_flags() {
        assert_eq!(extra_flags(&[0x02u8][..]), Done(empty!(), ExtraFlags::MaximumCompression));
        assert_eq!(extra_flags(&[0x04u8][..]), Done(empty!(), ExtraFlags::FastestAlgorithm));
        for byte in 0x00u8 .. 0xffu8 {
            let masked = byte & 0b1111_1001;
            assert_eq!(extra_flags(&[masked][..]), Done(empty!(), ExtraFlags::Unknown));
        }
    }

    #[test]
    fn test_operating_system() {
        use OperatingSystem::*;
        assert_eq!(operating_system(&[0u8][..]),  Done(empty!(), Fat));
        assert_eq!(operating_system(&[1u8][..]),  Done(empty!(), Amiga));
        assert_eq!(operating_system(&[2u8][..]),  Done(empty!(), Vms));
        assert_eq!(operating_system(&[3u8][..]),  Done(empty!(), Unix));
        assert_eq!(operating_system(&[4u8][..]),  Done(empty!(), VmCms));
        assert_eq!(operating_system(&[5u8][..]),  Done(empty!(), AtariTos));
        assert_eq!(operating_system(&[6u8][..]),  Done(empty!(), Hpfs));
        assert_eq!(operating_system(&[7u8][..]),  Done(empty!(), Macintosh));
        assert_eq!(operating_system(&[8u8][..]),  Done(empty!(), Zsystem));
        assert_eq!(operating_system(&[9u8][..]),  Done(empty!(), Cpm));
        assert_eq!(operating_system(&[10u8][..]), Done(empty!(), Tops20));
        assert_eq!(operating_system(&[11u8][..]), Done(empty!(), Ntfs));
        assert_eq!(operating_system(&[12u8][..]), Done(empty!(), Qdos));
        assert_eq!(operating_system(&[13u8][..]), Done(empty!(), AcornRiscos));
        for b in 14u8 .. 0xffu8 {
            assert_eq!(operating_system(&[b][..]), Done(empty!(), Unknown));
        }
    }

    #[test]
    fn test_sub_field() {
        use tests::byteorder::{ByteOrder, LittleEndian};
        let mut field: [u8; 8] = [0; 8];
        for (pos, val) in "cp  cpio".bytes().enumerate() {
            field[pos] = val;
        }
        LittleEndian::write_u16(&mut field[2..4], 4);

        assert_eq!(sub_field(&field[..]), Done(empty!(), SubField {
            id1: 'c' as u8,
            id2: 'p' as u8,
            data: &b"cpio"[..],
        }));
    }

    #[test]
    fn test_extra_field() {
        use tests::byteorder::{ByteOrder, LittleEndian};
        let mut xfield: [u8; 42] = [0; 42];
        for (pos, val) in "  cp  cpio.Ac  acorn.KN  keynote assertion".bytes().enumerate() {
            xfield[pos] = val;
        }
        LittleEndian::write_u16(&mut xfield[0..2],  40);
        LittleEndian::write_u16(&mut xfield[4..6],   5);
        LittleEndian::write_u16(&mut xfield[13..15], 6);
        LittleEndian::write_u16(&mut xfield[23..25], 17);

        match extra_field(&xfield[..]) {
            Done(_, actual) => {
                assert!(actual.sub_fields.contains(&SubField {
                    id1: 'c' as u8,
                    id2: 'p' as u8,
                    data: &b"cpio."[..],
                }));
                assert!(actual.sub_fields.contains(&SubField {
                    id1: 'A' as u8,
                    id2: 'c' as u8,
                    data: &b"acorn."[..],
                }));
                assert!(actual.sub_fields.contains(&SubField {
                    id1: 'K' as u8,
                    id2: 'N' as u8,
                    data: &b"keynote assertion"[..],
                }));

            },
            unexpected => assert!(false, "Unable to parse extra field, got back {:?}", unexpected),
        }
    }

    #[test]
    fn test_get_byte() {
        for expected in 0x00u8 .. 0xffu8 {
            assert_eq!(get_byte(&[expected][..]), Done(empty!(), expected));
        }
    }

    #[test]
    fn test_null_terminated_string() {
        test_null_terminated!(null_terminated_string);
    }

    #[test]
    fn test_original_filename() {
        test_null_terminated!(original_filename);
    }

    #[test]
    fn test_file_comment() {
        test_null_terminated!(file_comment);
    }

    #[test]
    fn test_header_crc16() {
        test_u16!(header_crc16);
    }

    #[test]
    fn test_footer_crc32() {
        test_u32!(footer_crc32);
    }

    #[test]
    fn test_input_size() {
        test_u32!(input_size);
    }

}
