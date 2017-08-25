# nom-gzip

[nom](https://github.com/Geal/nom) parser for the GZIP file format,
as documented in [RFC 1952](https://tools.ietf.org/rfc/rfc1952.txt).

# Installation

[nom-gzip](https://github.com/Geal/nom) is available on [crates.io](https://crates.io/crates/nom-gzip) and can be used in your project by adding the following to your `Cargo.toml` file:

    [dependencies]
    nom-gzip = "0.1.0"

# Usage

Three functions are available:

* `gzip_file`
* `gzip_header`
* `gzip_footer`

Once the GZIP header has been parsed, the remaining data are the compressed
blocks and an 8-byte footer. If using a seekable stream it's recommended
to parse the header with `gzip_header`, grab the remaining bytes - minus
the 8 at the end - as the compressed blocks, then call `parse_footer`
on the remaining 8 bytes. This should be considerably faster than parsing
byte-by-byte looking for the end of stream.

# Notes on this parser

## TL;DR

This parser assumes the GZIP stream contains only a single compressed file
that goes until EOF.

## Details

While in theory multiple files can be in a single GZIP stream
by simply concatenating multiple GZIP files together (see [section
2.2](https://tools.ietf.org/html/rfc1952#page-5]) of the RFC), in practice
it appears that at least GNU GZIP and 7z do not correctly support this. For
two files cat'd together they both report the header of the first file with
the footer (uncompressed size of the file) from the second. Decompression
of such a file with the gzip utility results in the uncompressed contents
of both files concatenated together in a single file instead of two files
with separated content. IMHO if this feature of the GZIP format can't be
used in any practical sense there is no point in spending time writing a
theoretically correct but far more involved (and slower!) parser here.
