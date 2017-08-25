#[derive(Debug, PartialEq)]
pub enum CompressionMethod {
    Reserved0,
    Reserved1,
    Reserved2,
    Reserved3,
    Reserved4,
    Reserved5,
    Reserved6,
    Reserved7,
    Deflate,
    Unknown
}

impl From<u8> for CompressionMethod {

    fn from(byte: u8) -> Self {
        use CompressionMethod::*;
        match byte {
            0 => Reserved0,
            1 => Reserved1,
            2 => Reserved2,
            3 => Reserved3,
            4 => Reserved4,
            5 => Reserved5,
            6 => Reserved6,
            7 => Reserved7,
            8 => Deflate,
            _ => Unknown,
        }
    }

}

#[derive(Debug, PartialEq)]
pub struct Flags {
    pub ftext:    bool,
    pub fhcrc:    bool,
    pub fextra:   bool,
    pub fname:    bool,
    pub fcomment: bool,
}

impl From<u8> for Flags {

    fn from(byte: u8) -> Self {
        Flags {
            ftext:    byte & 0b0000_0001 > 0,
            fhcrc:    byte & 0b0000_0010 > 0,
            fextra:   byte & 0b0000_0100 > 0,
            fname:    byte & 0b0000_1000 > 0,
            fcomment: byte & 0b0001_0000 > 0,
        }
    }

}

#[derive(Debug, PartialEq)]
pub enum ExtraFlags {
    MaximumCompression,
    FastestAlgorithm,
    Unknown,
}

impl From<u8> for ExtraFlags {

    fn from(byte: u8) -> Self {
        use ExtraFlags::*;
        match byte {
            2u8 => MaximumCompression,
            4u8 => FastestAlgorithm,
            _ => Unknown,
        }
    }

}

#[derive(Debug, PartialEq)]
pub enum OperatingSystem {
    Fat,
    Amiga,
    Vms,
    Unix,
    VmCms,
    AtariTos,
    Hpfs,
    Macintosh,
    Zsystem,
    Cpm,
    Tops20,
    Ntfs,
    Qdos,
    AcornRiscos,
    Unknown,
}

impl From<u8> for OperatingSystem {

    fn from(byte: u8) -> Self {
        use OperatingSystem::*;
        match byte {
            0  => Fat,
            1  => Amiga,
            2  => Vms,
            3  => Unix,
            4  => VmCms,
            5  => AtariTos,
            6  => Hpfs,
            7  => Macintosh,
            8  => Zsystem,
            9  => Cpm,
            10 => Tops20,
            11 => Ntfs,
            12 => Qdos,
            13 => AcornRiscos,
               _ => Unknown,
        }
    }

}

#[derive(Debug, PartialEq)]
pub struct SubField<'a> {
    pub id1: u8,
    pub id2: u8,
    pub data: &'a[u8],
}

#[derive(Debug, PartialEq)]
pub struct ExtraField<'a> {
    pub sub_fields: Vec<SubField<'a> >,
}

#[derive(Debug)]
pub struct GzipHeader<'a> {
    pub compression_method: CompressionMethod,
    pub flags: Flags,
    pub modified_time_as_secs_since_epoch: ::std::time::Duration,
    pub extra_flags: ExtraFlags,
    pub operating_system: OperatingSystem,
    pub extra_field: Option<ExtraField<'a>>,
    pub original_filename: Option<String>,
    pub file_comment: Option<String>,
    pub header_crc: Option<u16>,
}

#[derive(Debug)]
pub struct GzipFooter {
    pub crc: u32,
    pub input_size: u32,
}

#[derive(Debug)]
pub struct GzipFile<'a> {
    pub header: GzipHeader<'a>,
    pub footer: GzipFooter,
    pub compressed_blocks: Vec<u8>,
}

#[cfg(test)]
mod tests {

use super::*;

    #[test]
    fn compression_method() {
        use CompressionMethod::*;
        assert_eq!(CompressionMethod::from(0), Reserved0);
        assert_eq!(CompressionMethod::from(1), Reserved1);
        assert_eq!(CompressionMethod::from(2), Reserved2);
        assert_eq!(CompressionMethod::from(3), Reserved3);
        assert_eq!(CompressionMethod::from(4), Reserved4);
        assert_eq!(CompressionMethod::from(5), Reserved5);
        assert_eq!(CompressionMethod::from(6), Reserved6);
        assert_eq!(CompressionMethod::from(7), Reserved7);
        assert_eq!(CompressionMethod::from(8), Deflate);
        for b in 9 .. u8::max_value() {
            assert_eq!(CompressionMethod::from(b), Unknown);
        }
    }

    #[test]
    fn flags() {
        for byte in 0b0000_0000 .. 0b0001_1111 {
            let expected = Flags {
                ftext:    byte & 0b0000_0001 > 0,
                fhcrc:    byte & 0b0000_0010 > 0,
                fextra:   byte & 0b0000_0100 > 0,
                fname:    byte & 0b0000_1000 > 0,
                fcomment: byte & 0b0001_0000 > 0,
            };
            let actual = Flags::from(byte);
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn xtra_flags() {
        use ExtraFlags::*;
        assert_eq!(ExtraFlags::from(2), MaximumCompression);
        assert_eq!(ExtraFlags::from(4), FastestAlgorithm);
        let mask: u8 = 0b1111_1001;
        for b in u8::min_value() .. u8::max_value() {
            assert_eq!(ExtraFlags::from(b & mask), Unknown);
        }
    }

    #[test]
    fn operating_system() {
        use OperatingSystem::*;
        assert_eq!(OperatingSystem::from(0),  Fat);
        assert_eq!(OperatingSystem::from(1),  Amiga);
        assert_eq!(OperatingSystem::from(2),  Vms);
        assert_eq!(OperatingSystem::from(3),  Unix);
        assert_eq!(OperatingSystem::from(4),  VmCms);
        assert_eq!(OperatingSystem::from(5),  AtariTos);
        assert_eq!(OperatingSystem::from(6),  Hpfs);
        assert_eq!(OperatingSystem::from(7),  Macintosh);
        assert_eq!(OperatingSystem::from(8),  Zsystem);
        assert_eq!(OperatingSystem::from(9),  Cpm);
        assert_eq!(OperatingSystem::from(10), Tops20);
        assert_eq!(OperatingSystem::from(11), Ntfs);
        assert_eq!(OperatingSystem::from(12), Qdos);
        assert_eq!(OperatingSystem::from(13), AcornRiscos);
        for b in 14u8 .. u8::max_value() {
            assert_eq!(OperatingSystem::from(b), Unknown);
        }
    }

}
