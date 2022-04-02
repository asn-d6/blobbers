/// A naive packer that packs 31bytes (248 bits) per field element.

use thiserror::Error;
use rand::Rng;

/// The number of field elements per blob
const FIELD_ELEMENTS_PER_BLOB: usize = 1016;
/// Max number of blobs per transaction
const MAX_BLOBS_PER_TX: usize = 2;
/// Bytes per field element (including useless part of field element)
const BYTES_PER_FIELD_ELEMENT: usize = 32;
/// Number of useful bytes we can fit into a field element (the rest need to be zero to fit into the modulus)
const USEFUL_BYTES_PER_FIELD_ELEMENT: usize = 31;

/// The number of useful bytes of data we can fit into one blob
const USEFUL_BYTES_PER_BLOB: usize = USEFUL_BYTES_PER_FIELD_ELEMENT * FIELD_ELEMENTS_PER_BLOB;
/// The max amount of useful bytes we can fit into one transaction (one byte is used as the padding separator)
pub const MAX_USEFUL_BYTES_PER_TX: usize = (USEFUL_BYTES_PER_BLOB * MAX_BLOBS_PER_TX) - 1;
/// The actual size of a blob (including the useless part of field elements)
const BLOB_SIZE: usize = BYTES_PER_FIELD_ELEMENT * FIELD_ELEMENTS_PER_BLOB;

/// A blob (just a bunch of bytes really...)
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
fn get_padded(data: &[u8], n_blobs: usize) -> Vec<u8> {
    // Create the padded vector
    let mut padded_data = vec![0; n_blobs*USEFUL_BYTES_PER_BLOB];

    padded_data[..data.len()].clone_from_slice(data);
    // XXX bugs if exactly the right amount of data
    padded_data[data.len()] = 0x80;

    return padded_data
}

/// Build and return a blob from arbitrary data
fn get_blob(data: &[u8; USEFUL_BYTES_PER_BLOB]) -> Blob {
    let mut blob = [0; BLOB_SIZE];

    // Start packing!  Data needs to be encoded as valid field elements to be a blob.
    for i in 0..FIELD_ELEMENTS_PER_BLOB {
        // Each field element is 32 bytes long, but only the first 31 bytes are used for actual data
        let mut chunk = vec![0; 32];
        // Copy data into the first 31 bytes
        chunk[..31].clone_from_slice(&data[i*31..(i+1)*31]);
        // Copy the entire 32 bytes into the blob
        blob[i*32..(i+1)*32].clone_from_slice(&chunk);
    }

//    println!("[*] New blob: {:?}", blob);
    return blob
}

/// Given the data in an array, return a list of blobs
pub fn get_blobs_from_data(data: &[u8]) -> Result<Vec<Blob>, PackingError> {
    if data.len() == 0 {
        println!("[!] Got no data as input. Exiting without doing any work.");
        return Err(PackingError::DataLengthError);
    }

    if data.len() > MAX_USEFUL_BYTES_PER_TX {
        println!("[!] You provided {} bytes, but we can only pack {} bytes into a single tx. Aborting!", data.len(), MAX_USEFUL_BYTES_PER_TX);
        return Err(PackingError::DataLengthError);
    }

    assert!(data.len() <= MAX_USEFUL_BYTES_PER_TX);

    let n_blobs_needed = data.len().div_ceil(USEFUL_BYTES_PER_BLOB); // XXX need nightly for div_ceil()
//    println!("[*] We got {} bytes, we will need {} blobs for that!", data.len(), n_blobs_needed);

    let padded_data = get_padded(data, n_blobs_needed);
//    println!("[*] We started with {} bytes; now we have {} padded bytes [{:?}]!", data.len(), padded_data.len(), padded_data);

    let mut blobs = Vec::<Blob>::with_capacity(n_blobs_needed);
    for i in 0..n_blobs_needed {
        // Get a bunch of data, and pack it into a blob
        let chunk = &padded_data[i*USEFUL_BYTES_PER_BLOB..(i+1)*USEFUL_BYTES_PER_BLOB];
        let blob = get_blob(chunk.try_into().expect("bad chunking"));
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

    fn clean_field_elements(data: &mut Vec<u8>) {
        // Trim the last byte out of every field element (it's forced to zero)
        let mut index = 0;
        data.retain(|_| {
            index += 1;
            index % 32 != 0
        });
    }

    #[test]
    fn pack() {
        let data: Vec<u8> = (0..USEFUL_BYTES_PER_BLOB + 5).map(|_| { rand::random::<u8>() }).collect();
        let blobs = get_blobs_from_data(&data).unwrap();

        let rcved_blob_data = blobs.concat();
        assert_eq!(rcved_blob_data.len(), blobs.len() * BLOB_SIZE);

//        println!("[?] Concatenated {:?}", rcved_blob_data);

        // Remove the padding
        let mut rcved_data = unpad(rcved_blob_data).unwrap();
        clean_field_elements(&mut rcved_data);

//        println!("[?] Unpadded {:?}", rcved_data);
        assert_eq!(rcved_data, data)
    }
}
