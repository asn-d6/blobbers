/// A tight packer. Packs 254 bits of data into each field element. Vroom vroom.

use thiserror::Error;
use rand::Rng;
use bitvec::prelude::*;

/// Max number of blobs per transaction
const MAX_BLOBS_PER_TX: usize = 2;
/// The number of field elements per blob
const FIELD_ELEMENTS_PER_BLOB: usize = 4096;
/// The number of useful bytes of data we can fit into one blob
const USEFUL_BYTES_PER_TIGHT_BLOB: usize = (254 * FIELD_ELEMENTS_PER_BLOB) / 8; // 254 bits per field element
/// The max amount of useful bytes we can fit into one transaction (one byte is used as the padding separator)
pub const MAX_TIGHT_USEFUL_BYTES_PER_TX: usize = (USEFUL_BYTES_PER_TIGHT_BLOB * MAX_BLOBS_PER_TX) - 1;
/// Bytes per field element on the wire
const BYTES_PER_FIELD_ELEMENT: usize = 32;
/// The actual size of a blob on the wire (including the useless part of field elements)
const BLOB_SIZE: usize = BYTES_PER_FIELD_ELEMENT * FIELD_ELEMENTS_PER_BLOB; // 32512

/// A blob on the wire (just a bunch of bytes really...)
type Blob = [u8; BLOB_SIZE];

////////////////////////////////////////////////////////////////////////////////

#[derive(Error, Debug)]
pub enum PackingError {
    #[error("Bad data length")]
    DataLengthError,
    #[error("Failed to unpad")]
    UnpadError,
}


/// Pad `data` to the right size to fit in `n_blobs`
fn get_padded_tight(data: &[u8], n_blobs: usize) -> Vec<u8> {
    // Create the padded vector
    let mut padded_data = vec![0; n_blobs*USEFUL_BYTES_PER_TIGHT_BLOB];

    padded_data[..data.len()].clone_from_slice(data);
    // XXX bugs if provided exactly the right amount of data
    padded_data[data.len()] = 0x80;

    return padded_data
}

fn get_blob_tight(data: &[u8; USEFUL_BYTES_PER_TIGHT_BLOB]) -> Blob {
    let mut blob = [0; BLOB_SIZE];

    let bits = BitSlice::<_, Msb0>::try_from_slice(data).unwrap();
    let iter = bits.chunks(254);
    for (i, chunk) in iter.enumerate() {
        let mut buf = [0; BYTES_PER_FIELD_ELEMENT];
        let buf_slice = buf.view_bits_mut::<Msb0>();
        buf_slice[..chunk.len()].copy_from_bitslice(chunk);

        blob[i*BYTES_PER_FIELD_ELEMENT..(i+1)*BYTES_PER_FIELD_ELEMENT].clone_from_slice(&buf);
    }

    return blob
}

/// Given the data in an array, return a list of blobs
pub fn get_tight_blobs_from_data(data: &[u8]) -> Result<Vec<Blob>, PackingError> {
    if data.len() == 0 {
        println!("[!] Got no data as input. Exiting without doing any work.");
        return Err(PackingError::DataLengthError);
    }

    if data.len() > MAX_TIGHT_USEFUL_BYTES_PER_TX {
        println!("[!] You provided {} bytes, but we can only pack {} bytes into a single tx. Aborting!", data.len(), MAX_TIGHT_USEFUL_BYTES_PER_TX);
        return Err(PackingError::DataLengthError);
    }

    assert!(data.len() <= MAX_TIGHT_USEFUL_BYTES_PER_TX);

    let n_blobs_needed = data.len().div_ceil(USEFUL_BYTES_PER_TIGHT_BLOB); // XXX need nightly for div_ceil()
//    println!("[*] We got {} bytes, we will need {} blobs for that!", data.len(), n_blobs_needed);

    let padded_data = get_padded_tight(data, n_blobs_needed);
//    println!("[*] We started with {} bytes; after padding we have {} bytes!", data.len(), padded_data.len());

    let mut blobs = Vec::<Blob>::with_capacity(n_blobs_needed);
    for i in 0..n_blobs_needed {
        // Get a bunch of data, and pack it into a blob
        let chunk = &padded_data[i*USEFUL_BYTES_PER_TIGHT_BLOB..(i+1)*USEFUL_BYTES_PER_TIGHT_BLOB];
        let blob = get_blob_tight(chunk.try_into().expect("bad chunking"));
//        println!("[*] Got {}th blob: {} bytes", i, blob.len());
        blobs.push(blob)
    }


    return Ok(blobs);
}


#[cfg(test)]
mod tests {
    use super::*;

    fn unpad(data: Vec<u8>) -> Result<Vec<u8>, PackingError> {
        for i in (0..data.len()).rev() {
            match data[i] {
                0x80 => return Ok(data[..i].to_vec()),
                0x00 => continue,
                _ => return Err(PackingError::UnpadError),
            }
        }
        Err(PackingError::UnpadError)
    }

    fn clean_field_elements_tight(data: &mut Vec<u8>) -> Vec<u8> {
        // Trim the last two bits out of every field element (they are forced to zero)

        let mut bitvec = BitVec::<_, Msb0>::from_slice(data);
        bitvec.retain(|idx, _| idx % 256 < 254); // remove padding
        return bitvec.into_vec()
    }


    #[test]
    fn pack_and_unpack() {
        let data: Vec<u8> = (0..USEFUL_BYTES_PER_TIGHT_BLOB - 5).map(|_| { rand::random::<u8>() }).collect();
        let blobs = get_tight_blobs_from_data(&data).unwrap();

        let mut rcved_blob_data = blobs.concat();
        assert_eq!(rcved_blob_data.len(), blobs.len() * BLOB_SIZE);

        let cleaned = clean_field_elements_tight(&mut rcved_blob_data);

        // Remove the padding
        let rcved_data = unpad(cleaned).unwrap();

        assert_eq!(rcved_data, data);
    }
}
