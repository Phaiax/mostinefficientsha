//! Encoding helpers between ascii, u32 and f64 representations of 256 bit hashes.

use data_encoding::hex;
use byteorder::{BigEndian, WriteBytesExt, ReadBytesExt};
use arrayvec::ArrayVec;

/// Hex encodes eight u32 numbers to a big-endian 64 characters lowercase 0-9a-f hash representation.
///
/// Panics if input data has wrong len.
pub fn hex(input_data : &[u32]) -> String {
    assert_eq!(input_data.len(), 8);
    // input_data has system endianess. On intel systems this is little endian.
    // So rewrite the bytes in big endian format for hex out.
    let mut bytes_bigendian = vec![];
    for i in input_data {
        bytes_bigendian.write_u32::<BigEndian>(*i).unwrap();
    }
    // Encode to hex using data_encoding crate
    hex::encode(&bytes_bigendian).to_lowercase()
}

/// Parses a big-endian 64 characters 0-9a-f hash representation to eight u32 numbers in system endianess.
///
/// Panics if input data has wrong len.
pub fn dehex(d : &str) -> Box<[u32; 8]> {
    assert_eq!(d.len(), 64);
    // Decode hex to bytes using data_encoding crate
    let bytes_bigendian = hex::decode(d.to_uppercase().as_bytes()).unwrap();
    let mut output_data = Box::<[u32;8]>::new( [0; 8]);
    // Rewrite the bytes to system endianess
    let mut reader_bytes_be = &bytes_bigendian[..];
    for d in output_data.iter_mut() {
        *d = reader_bytes_be.read_u32::<BigEndian>().unwrap();
    }
    output_data
}

/// Takes 256 bits where each bit is represented by a float.
///
/// Rounds these bits and assembles them to eight u32 numbers.
///
/// The input data is big endian, with i[0] being the MSBit.
///
/// Panics if input data has wrong len.
pub fn f64bits_to_u32(i : &[f64]) -> ArrayVec<[u32; 8]> {
    assert_eq!(i.len(), 256);
    let mut out = ArrayVec::<[u32; 8]>::new();
    for i_chunk in i.chunks(32) {
        let mut o = 0;
        for b in i_chunk {
            o = o << 1; // note: first shift does nothing, intended
            if *b >= 0.5 {
                o = o | 1u32;
            }
        }
        out.push(o);
    }
    out
}

/// The inverse of `f64bits_to_u32()`. Creates 256 doubles
/// that are either 1.0 or 0.0 . return_value[0] is the MSBit.
///
/// Panics if input data has wrong len.
pub fn u32_to_f64bits(hex : &[u32]) -> Vec<f64> {
    assert_eq!(hex.len(), 8);
    let mut out = Vec::with_capacity(256);
    for i in hex.iter() {
        let mut c : u32 = *i;
        for _ in 0..32 {
            out.push( if c & 0b10000000_00000000_00000000_00000000_u32 != 0u32 { 1.0 } else { 0.0 } );
            c = c << 1;
        }
    }
    out
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_dehex() {
        use std::ops::Deref;
        assert_eq!(hex(&dehex("ca978112ca1bbdcafac231b39a23dc4da786eff8147c4e72b9807785afee48bb").deref()[..]),
                   "ca978112ca1bbdcafac231b39a23dc4da786eff8147c4e72b9807785afee48bb" );
    }

    #[test]
    fn f64conversion() {
        let almost1 = (0..256).map(|_| 0.9f64).collect::<Vec<_>>();
        let ffff : Vec<u32> = (0..8).map(|_| u32::max_value()).collect();
        assert_eq!(f64bits_to_u32(almost1.as_ref()).as_ref(), &ffff[..]);

        let almost0 = (0..256).map(|_| 0.1f64).collect::<Vec<_>>();
        let zzzz : Vec<u32> = (0..8).map(|_| u32::min_value()).collect();
        assert_eq!(f64bits_to_u32(almost0.as_ref()).as_ref(), &zzzz[..]);

        let arbitrary : Vec<u32> = (0..8).map(|i| i*3).collect();
        let arbitrary_f64 = u32_to_f64bits(&arbitrary[..]);
        assert_eq!(f64bits_to_u32(arbitrary_f64.as_ref()).as_ref(), &arbitrary[..]);
    }


}
