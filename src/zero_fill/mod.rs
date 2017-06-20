pub mod error;

use self::error::Error;
use hound::{WavReader, WavWriter};

use std::ffi::CStr;
use std::fs::{self, File};
use std::io::Write;
use std::os::raw::c_char;
use std::path::Path;

const ZERO_FILL_OK: i32 = 0;
const ZERO_FILL_C_STR_CONV: i32 = 1;
const ZERO_FILL_FILENAME: i32 = 2;
const ZERO_FILL_IO: i32 = 3;
const ZERO_FILL_WAV: i32 = 4;

#[no_mangle]
pub extern fn zero_fill_matching(file_path: *const c_char) -> i32 {
    let file_path = unsafe { CStr::from_ptr(file_path).to_str() };

    let res = match file_path {
        Ok(file_path) => fill_matching(file_path),
        Err(e) => Err(Error::CStrConv(e)),
    };

    match res {
        Ok(_) => ZERO_FILL_OK,
        Err(e) => match e {
            Error::CStrConv(_) => ZERO_FILL_C_STR_CONV,
            Error::Filename => ZERO_FILL_FILENAME,
            Error::IO(_) => ZERO_FILL_IO,
            Error::Wav(_) => ZERO_FILL_WAV,
        },
    }
}

enum ExtType {
    Wav,
    Others,
}

pub fn fill_matching<P: AsRef<Path>>(file_path: P) -> error::Result<()> {
    let file_path = file_path.as_ref();

    let ext_type = match file_path.extension() {
        Some(ext) => {
            let ext = ext.to_string_lossy().to_lowercase();

            match ext.as_ref() {
                "wav" => ExtType::Wav,
                _ => ExtType::Others,
            }
        },

        None => ExtType::Others,
    };

    match ext_type {
        ExtType::Wav => fill_wav(file_path),
        ExtType::Others => fill_any(file_path),
    }
}

pub fn fill_wav<P: AsRef<Path>>(file_path: P) -> error::Result<()> {
    let file_path = file_path.as_ref();

    let (spec, len) = {
        let reader = WavReader::open(file_path)?;
        (reader.spec().clone(), reader.len())
    };

    let mut writer = WavWriter::create(file_path, spec)?;

    for _ in 0..len {
        writer.write_sample(0 as i16)?;
    }
    
    Ok(writer.finalize()?)
}

pub fn fill_any<P: AsRef<Path>>(file_path: P) -> error::Result<()> {
    let file_path = file_path.as_ref();

    let metadata = fs::metadata(file_path)?;
    let len = metadata.len();

    let mut file = File::create(file_path)?;
    Ok(file.write_all(&vec![0u8; len as usize])?)
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs;
    use std::io::Read;

    fn wav_vec() -> Vec<u8> {
        vec![0x52, 0x49, 0x46, 0x46, 0x06, 0x0F, 0x00, 0x00, 0x57, 0x41, 0x56, 0x45, 0x66, 0x6D, 0x74, 0x20, 0x10, 0x00, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x11, 0x2B, 0x00, 0x00, 0x11, 0x2B, 0x00, 0x00, 0x01, 0x00, 0x08, 0x00, 0x64, 0x61, 0x74, 0x61, 0xE1, 0x0E, 0x00, 0x00, 0x81, 0x81, 0x80, 0x82, 0x80, 0x82, 0x80, 0x82, 0x81, 0x82, 0x82, 0x82, 0x82, 0x81, 0x82, 0x81, 0x82, 0x81, 0x82, 0x81, 0x82, 0x81, 0x81, 0x82, 0x81, 0x82, 0x81, 0x83, 0x81, 0x83, 0x81, 0x82, 0x82, 0x82, 0x83, 0x82, 0x83, 0x81, 0x82, 0x81, 0x83, 0x82, 0x84, 0x83, 0x84, 0x85, 0x85, 0x86, 0x86, 0x88, 0x86, 0x86, 0x83, 0x84, 0x81, 0x81, 0x80, 0x7E, 0x7E, 0x7D, 0x7E, 0x7D, 0x7F, 0x7E, 0x7F, 0x7F, 0x7F, 0x80, 0x80, 0x82, 0x81, 0x83, 0x83, 0x84, 0x84, 0x84, 0x85, 0x84, 0x86, 0x86, 0x87, 0x87, 0x88, 0x88, 0x87, 0x89, 0x87, 0x89, 0x87, 0x88, 0x88, 0x87, 0x87, 0x85, 0x85, 0x83, 0x84, 0x83, 0x83, 0x83, 0x81, 0x83, 0x82, 0x84, 0x82, 0x83, 0x83, 0x83, 0x83, 0x81, 0x82, 0x82, 0x84, 0x81, 0x81, 0x81, 0x82, 0x84, 0x83, 0x85, 0x84, 0x86, 0x87, 0x8A, 0x8D, 0x8E, 0x8F, 0x8B, 0x89, 0x85, 0x85, 0x82, 0x80, 0x7E, 0x7C, 0x7D, 0x7B, 0x7D, 0x7C, 0x7E, 0x7E, 0x7F, 0x7F, 0x80, 0x82, 0x81, 0x83, 0x82, 0x83, 0x82, 0x82, 0x83, 0x83, 0x85, 0x84, 0x86, 0x86, 0x8A, 0x8A, 0x8D, 0x8D, 0x8C, 0x8C, 0x8A, 0x8B, 0x89, 0x89, 0x86, 0x84, 0x80, 0x7F, 0x7F, 0x7F, 0x7F, 0x7C, 0x7D, 0x7B, 0x7E, 0x7E, 0x81, 0x82, 0x81, 0x81, 0x81, 0x84, 0x84, 0x85, 0x82, 0x81, 0x80, 0x83, 0x83, 0x84, 0x87, 0x89, 0x8C, 0x8C, 0x92, 0x95, 0x99, 0x95, 0x92, 0x8C, 0x89, 0x86, 0x81, 0x7F, 0x7C, 0x7C, 0x7B, 0x7D, 0x7C, 0x7D, 0x7B, 0x7D, 0x7D, 0x7E, 0x7F, 0x7F, 0x81, 0x80, 0x82, 0x81, 0x83, 0x83, 0x85, 0x84, 0x86, 0x87, 0x89, 0x8B, 0x8B, 0x8C, 0x8B, 0x8C, 0x8A, 0x8B, 0x88, 0x88, 0x86, 0x84, 0x83, 0x80, 0x80, 0x7D, 0x7D, 0x7C, 0x7D, 0x7D, 0x7F, 0x80, 0x80, 0x81, 0x7F, 0x81, 0x81, 0x84, 0x85, 0x83, 0x81, 0x7E, 0x7F, 0x80, 0x83, 0x85, 0x89, 0x8F, 0x92, 0x97, 0x99, 0x9D, 0x9A, 0x95, 0x8E, 0x88, 0x84, 0x7E, 0x7C, 0x78, 0x79, 0x79, 0x7A, 0x7B, 0x7A, 0x7A, 0x78, 0x78, 0x77, 0x78, 0x78, 0x78, 0x7B, 0x7C, 0x80, 0x80, 0x84, 0x85, 0x87, 0x88, 0x89, 0x8C, 0x8C, 0x8E, 0x8D, 0x8E, 0x8F, 0x8F, 0x8E, 0x8A, 0x8A, 0x87, 0x87, 0x83, 0x83, 0x81, 0x7E, 0x7D, 0x7B, 0x7D, 0x7D, 0x80, 0x7F, 0x7E, 0x7D, 0x7D, 0x7F, 0x7F, 0x80, 0x7E, 0x80, 0x81, 0x85, 0x89, 0x8A, 0x8D, 0x8F, 0x95, 0x98, 0x9C, 0x9B, 0x98, 0x92, 0x8B, 0x86, 0x80, 0x7D, 0x78, 0x76, 0x74, 0x74, 0x73, 0x73, 0x75, 0x77, 0x7A, 0x78, 0x79, 0x79, 0x7C, 0x7C, 0x7E, 0x81, 0x84, 0x89, 0x89, 0x8D, 0x8D, 0x8F, 0x8C, 0x8D, 0x8D, 0x8E, 0x8E, 0x8D, 0x8E, 0x8C, 0x8A, 0x85, 0x87, 0x85, 0x82, 0x7D, 0x7B, 0x7A, 0x78, 0x7A, 0x7B, 0x7F, 0x7D, 0x7B, 0x78, 0x78, 0x77, 0x74, 0x72, 0x74, 0x7B, 0x81, 0x88, 0x8B, 0x93, 0x97, 0x9C, 0x9E, 0x9E, 0x9C, 0x93, 0x8B, 0x83, 0x80, 0x7C, 0x7D, 0x7C, 0x7C, 0x79, 0x79, 0x78, 0x75, 0x74, 0x73, 0x76, 0x76, 0x79, 0x7A, 0x7E, 0x80, 0x82, 0x83, 0x84, 0x88, 0x8A, 0x8F, 0x92, 0x96, 0x96, 0x96, 0x95, 0x92, 0x8F, 0x89, 0x87, 0x84, 0x84, 0x83, 0x81, 0x81, 0x80, 0x80, 0x7E, 0x80, 0x80, 0x7E, 0x7A, 0x77, 0x76, 0x73, 0x72, 0x6E, 0x70, 0x74, 0x7B, 0x82, 0x87, 0x90, 0x95, 0x9A, 0x99, 0x9C, 0x9C, 0x96, 0x8E, 0x86, 0x83, 0x7F, 0x7E, 0x7C, 0x7C, 0x7C, 0x7B, 0x7B, 0x79, 0x7A, 0x77, 0x76, 0x74, 0x75, 0x78, 0x78, 0x7A, 0x7C, 0x81, 0x82, 0x87, 0x8C, 0x8E, 0x8E, 0x8B, 0x8C, 0x88, 0x85, 0x82, 0x85, 0x88, 0x87, 0x88, 0x89, 0x8A, 0x84, 0x81, 0x7B, 0x7B, 0x79, 0x79, 0x7B, 0x7F, 0x85, 0x85, 0x85, 0x82, 0x7E, 0x78, 0x74, 0x76, 0x7A, 0x84, 0x8A, 0x94, 0x99, 0x9E, 0x9C, 0x94, 0x8B, 0x83, 0x7F, 0x7C, 0x7F, 0x7F, 0x7D, 0x79, 0x7A, 0x7A, 0x79, 0x7B, 0x7C, 0x7F, 0x7E, 0x80, 0x80, 0x82, 0x82, 0x82, 0x80, 0x83, 0x88, 0x8A, 0x8B, 0x8B, 0x8F, 0x90, 0x90, 0x8C, 0x8A, 0x87, 0x80, 0x7C, 0x7C, 0x83, 0x81, 0x7E, 0x7D, 0x7F, 0x78, 0x70, 0x6F, 0x76, 0x7C, 0x81, 0x8F, 0x9C, 0x9A, 0x84, 0x79, 0x77, 0x70, 0x62, 0x6D, 0x90, 0xA8, 0xA5, 0x97, 0x91, 0x88, 0x81, 0x7C, 0x7B, 0x75, 0x70, 0x6C, 0x6E, 0x71, 0x78, 0x82, 0x86, 0x85, 0x7D, 0x7B, 0x78, 0x75, 0x76, 0x83, 0x95, 0xA1, 0xA5, 0x9F, 0x98, 0x89, 0x7D, 0x75, 0x75, 0x7D, 0x8A, 0x96, 0x96, 0x96, 0x8C, 0x7A, 0x65, 0x5E, 0x68, 0x73, 0x85, 0x9A, 0xAA, 0x9E, 0x83, 0x6E, 0x64, 0x5A, 0x52, 0x6B, 0x98, 0xB5, 0xAA, 0x96, 0x8E, 0x8A, 0x85, 0x7D, 0x79, 0x75, 0x75, 0x77, 0x79, 0x7E, 0x7E, 0x7E, 0x7A, 0x78, 0x77, 0x7B, 0x85, 0x8E, 0x95, 0x94, 0x94, 0x8F, 0x86, 0x7E, 0x7D, 0x85, 0x88, 0x8B, 0x8E, 0x92, 0x8C, 0x7C, 0x72, 0x74, 0x7B, 0x7A, 0x83, 0x97, 0xA4, 0x94, 0x77, 0x68, 0x64, 0x5E, 0x56, 0x6B, 0x93, 0xA8, 0x9E, 0x8B, 0x86, 0x80, 0x7C, 0x7A, 0x7C, 0x7A, 0x7A, 0x7D, 0x7F, 0x81, 0x7D, 0x7D, 0x7B, 0x7D, 0x7E, 0x82, 0x86, 0x8C, 0x8F, 0x89, 0x84, 0x7F, 0x83, 0x82, 0x80, 0x81, 0x88, 0x8D, 0x86, 0x81, 0x82, 0x87, 0x80, 0x7C, 0x85, 0x98, 0x9A, 0x87, 0x77, 0x74, 0x75, 0x6C, 0x71, 0x89, 0xA4, 0xA4, 0x96, 0x8C, 0x89, 0x86, 0x7F, 0x7C, 0x76, 0x76, 0x77, 0x7A, 0x7B, 0x7B, 0x7C, 0x7D, 0x7C, 0x7B, 0x81, 0x88, 0x8C, 0x87, 0x84, 0x82, 0x7F, 0x7B, 0x7A, 0x80, 0x85, 0x8B, 0x89, 0x88, 0x8A, 0x90, 0x88, 0x7E, 0x7C, 0x89, 0x91, 0x87, 0x79, 0x71, 0x70, 0x65, 0x60, 0x71, 0x95, 0xA8, 0xA2, 0x96, 0x8F, 0x89, 0x7F, 0x7B, 0x78, 0x7A, 0x7A, 0x7D, 0x7C, 0x7D, 0x7E, 0x7C, 0x7D, 0x80, 0x87, 0x89, 0x88, 0x85, 0x82, 0x7E, 0x75, 0x71, 0x74, 0x81, 0x86, 0x87, 0x86, 0x88, 0x89, 0x80, 0x7B, 0x7F, 0x8C, 0x8D, 0x83, 0x7F, 0x82, 0x81, 0x75, 0x79, 0x8E, 0xA1, 0x9D, 0x90, 0x89, 0x84, 0x7F, 0x78, 0x79, 0x7A, 0x77, 0x76, 0x78, 0x7C, 0x7A, 0x7C, 0x7D, 0x80, 0x81, 0x82, 0x82, 0x7D, 0x7A, 0x75, 0x77, 0x7C, 0x8A, 0x93, 0x8E, 0x88, 0x89, 0x8E, 0x89, 0x86, 0x87, 0x87, 0x7D, 0x6D, 0x6B, 0x73, 0x7A, 0x73, 0x7A, 0x91, 0xA0, 0x98, 0x89, 0x87, 0x81, 0x79, 0x76, 0x7A, 0x77, 0x71, 0x72, 0x74, 0x7A, 0x7F, 0x88, 0x8A, 0x87, 0x7C, 0x76, 0x7B, 0x80, 0x83, 0x86, 0x8F, 0x8E, 0x86, 0x7F, 0x84, 0x8A, 0x87, 0x7F, 0x7B, 0x80, 0x80, 0x7E, 0x7C, 0x80, 0x82, 0x86, 0x91, 0xA0, 0xA8, 0x96, 0x7C, 0x6C, 0x72, 0x78, 0x7D, 0x85, 0x88, 0x80, 0x74, 0x75, 0x79, 0x7F, 0x83, 0x86, 0x83, 0x82, 0x7E, 0x74, 0x6E, 0x75, 0x86, 0x8C, 0x8D, 0x95, 0x9D, 0x89, 0x71, 0x76, 0x8F, 0x93, 0x87, 0x85, 0x7D, 0x6C, 0x62, 0x73, 0x8D, 0xA4, 0x9E, 0x8C, 0x86, 0x88, 0x84, 0x7D, 0x84, 0x82, 0x73, 0x6A, 0x77, 0x81, 0x86, 0x87, 0x84, 0x7F, 0x78, 0x6C, 0x6E, 0x92, 0xAD, 0xA7, 0x91, 0x8A, 0x78, 0x63, 0x6D, 0x8C, 0x94, 0x81, 0x77, 0x70, 0x5F, 0x4D, 0x62, 0x9A, 0xC3, 0xBF, 0xAB, 0xA2, 0x8B, 0x69, 0x52, 0x5E, 0x7F, 0x9B, 0xA6, 0x95, 0x6F, 0x4F, 0x58, 0x74, 0x8C, 0xA7, 0xBA, 0xB1, 0x8A, 0x6F, 0x5A, 0x47, 0x5C, 0xA2, 0xCC, 0xAF, 0x87, 0x66, 0x30, 0x1D, 0x6B, 0xD5, 0xEA, 0xB9, 0x8A, 0x6B, 0x58, 0x64, 0x78, 0x7E, 0x80, 0x7E, 0x82, 0x79, 0x5D, 0x4C, 0x5C, 0x7A, 0xA2, 0xD3, 0xC6, 0x7C, 0x38, 0x29, 0x48, 0x97, 0xEB, 0xE0, 0x91, 0x4E, 0x25, 0x2B, 0x86, 0xEC, 0xE0, 0x8E, 0x6F, 0x7F, 0x84, 0x7F, 0x74, 0x66, 0x6C, 0x89, 0x9A, 0x76, 0x4C, 0x4A, 0x63, 0x98, 0xD9, 0xCF, 0x73, 0x2E, 0x28, 0x58, 0xB3, 0xF2, 0xC3, 0x6F, 0x45, 0x2C, 0x4F, 0xB5, 0xE9, 0xAD, 0x7C, 0x8D, 0x9D, 0x8F, 0x73, 0x5D, 0x61, 0x8B, 0xAB, 0x90, 0x52, 0x3C, 0x5D, 0xA4, 0xE1, 0xC1, 0x62, 0x23, 0x36, 0x7C, 0xD4, 0xE5, 0xA0, 0x5B, 0x2E, 0x34, 0x83, 0xD9, 0xC9, 0x8C, 0x8C, 0xA3, 0x9B, 0x7A, 0x55, 0x4E, 0x7D, 0xB3, 0xA8, 0x5A, 0x2A, 0x44, 0x97, 0xDB, 0xC8, 0x6A, 0x23, 0x34, 0x81, 0xD4, 0xE2, 0xA4, 0x55, 0x27, 0x3B, 0x99, 0xDC, 0xBA, 0x8E, 0x95, 0xA6, 0x96, 0x6C, 0x49, 0x62, 0xA3, 0xC1, 0x86, 0x33, 0x2E, 0x7B, 0xD0, 0xD5, 0x88, 0x30, 0x28, 0x6F, 0xC7, 0xE7, 0xAE, 0x59, 0x1D, 0x32, 0x97, 0xE0, 0xBB, 0x88, 0x91, 0xA9, 0x99, 0x5E, 0x36, 0x57, 0xAE, 0xC4, 0x74, 0x21, 0x30, 0x8D, 0xD2, 0xC5, 0x68, 0x22, 0x34, 0x8B, 0xDC, 0xDA, 0x8A, 0x34, 0x18, 0x59, 0xC8, 0xDC, 0x98, 0x7F, 0x9C, 0xAB, 0x7D, 0x3D, 0x37, 0x87, 0xCD, 0xA3, 0x44, 0x26, 0x6B, 0xC4, 0xD1, 0x84, 0x2A, 0x26, 0x77, 0xD5, 0xE0, 0x91, 0x32, 0x16, 0x5D, 0xCE, 0xDD, 0x8D, 0x76, 0xA2, 0xBB, 0x86, 0x30, 0x26, 0x78, 0xD4, 0xAE, 0x46, 0x20, 0x62, 0xBF, 0xD5, 0x86, 0x2C, 0x2D, 0x80, 0xDC, 0xDC, 0x80, 0x28, 0x23, 0x80, 0xDE, 0xC6, 0x7A, 0x85, 0xBA, 0xB0, 0x5D, 0x1E, 0x45, 0xAF, 0xD9, 0x7A, 0x24, 0x32, 0x8C, 0xD1, 0xB6, 0x57, 0x2A, 0x5C, 0xBB, 0xE4, 0xA4, 0x43, 0x1E, 0x5F, 0xC9, 0xD3, 0x8F, 0x85, 0xB7, 0xAE, 0x59, 0x20, 0x51, 0xBD, 0xDC, 0x7B, 0x21, 0x30, 0x8F, 0xDF, 0xC8, 0x62, 0x24, 0x4F, 0xB7, 0xEB, 0xB7, 0x4E, 0x17, 0x4B, 0xC3, 0xDE, 0xA5, 0x99, 0xB0, 0x83, 0x34, 0x2E, 0x86, 0xDD, 0xA6, 0x39, 0x17, 0x61, 0xC5, 0xD4, 0x7E, 0x29, 0x36, 0x94, 0xE4, 0xC6, 0x5A, 0x0E, 0x2C, 0xA3, 0xEE, 0xB4, 0x8C, 0x98, 0x7B, 0x3F, 0x49, 0xA4, 0xD2, 0x85, 0x1F, 0x1C, 0x72, 0xD2, 0xCE, 0x75, 0x24, 0x2E, 0x92, 0xE6, 0xCF, 0x5C, 0x08, 0x1C, 0x9B, 0xF7, 0xC3, 0x8B, 0x91, 0x79, 0x48, 0x68, 0xB8, 0xAE, 0x40, 0x04, 0x3D, 0xB0, 0xE5, 0x99, 0x2C, 0x10, 0x5F, 0xDD, 0xEC, 0x80, 0x0E, 0x04, 0x65, 0xEA, 0xEE, 0x92, 0x7D, 0x73, 0x5B, 0x7E, 0xB7, 0x94, 0x2B, 0x0E, 0x5C, 0xCF, 0xD9, 0x70, 0x12, 0x1E, 0x89, 0xEE, 0xDC, 0x61, 0x05, 0x14, 0x84, 0xF5, 0xDA, 0x7D, 0x69, 0x7E, 0x8E, 0xAC, 0x96, 0x45, 0x1A, 0x51, 0xBA, 0xDD, 0x92, 0x2C, 0x21, 0x76, 0xDB, 0xEB, 0x8D, 0x21, 0x11, 0x67, 0xDC, 0xF9, 0xA1, 0x5D, 0x79, 0xAD, 0xC0, 0x97, 0x4C, 0x28, 0x5A, 0xAD, 0xCC, 0x93, 0x40, 0x2F, 0x76, 0xCF, 0xDF, 0x9C, 0x37, 0x18, 0x5A, 0xBB, 0xE7, 0xA5, 0x6A, 0x95, 0xDE, 0xC4, 0x6F, 0x1D, 0x22, 0x7C, 0xC0, 0xB2, 0x7E, 0x47, 0x42, 0x83, 0xC1, 0xCA, 0x90, 0x3D, 0x29, 0x5B, 0xB2, 0xDC, 0xBC, 0xAA, 0xC3, 0xB1, 0x64, 0x22, 0x2E, 0x87, 0xC4, 0xA2, 0x57, 0x2C, 0x3D, 0x89, 0xCD, 0xCD, 0x8E, 0x32, 0x12, 0x45, 0xA9, 0xF5, 0xE0, 0xA6, 0x8D, 0x7B, 0x60, 0x65, 0x7A, 0x73, 0x50, 0x44, 0x54, 0x7C, 0xA4, 0xAF, 0x99, 0x6C, 0x45, 0x47, 0x73, 0xAB, 0xCD, 0xB3, 0x8E, 0x81, 0x7B, 0x70, 0x6B, 0x6B, 0x67, 0x67, 0x66, 0x66, 0x60, 0x62, 0x6E, 0x83, 0x96, 0x99, 0x91, 0x8A, 0x89, 0x83, 0x80, 0x7E, 0x7D, 0x7B, 0x7B, 0x7E, 0x7E, 0x80, 0x7F, 0x82, 0x84, 0x86, 0x88, 0x88, 0x8B, 0x8A, 0x89, 0x86, 0x88, 0x87, 0x86, 0x86, 0x83, 0x85, 0x83, 0x85, 0x84, 0x83, 0x83, 0x84, 0x86, 0x85, 0x88, 0x87, 0x89, 0x8A, 0x8C, 0x8C, 0x88, 0x88, 0x87, 0x88, 0x86, 0x86, 0x87, 0x87, 0x88, 0x87, 0x89, 0x86, 0x87, 0x87, 0x88, 0x88, 0x87, 0x88, 0x85, 0x85, 0x84, 0x87, 0x86, 0x86, 0x86, 0x85, 0x85, 0x84, 0x85, 0x82, 0x82, 0x82, 0x80, 0x7F, 0x7D, 0x7E, 0x7D, 0x7F, 0x7F, 0x80, 0x80, 0x7F, 0x7F, 0x7E, 0x81, 0x81, 0x83, 0x80, 0x80, 0x7F, 0x7D, 0x7C, 0x79, 0x79, 0x77, 0x79, 0x79, 0x7A, 0x79, 0x77, 0x76, 0x75, 0x77, 0x76, 0x76, 0x72, 0x71, 0x6E, 0x6D, 0x6D, 0x6D, 0x6E, 0x70, 0x74, 0x73, 0x73, 0x72, 0x73, 0x73, 0x74, 0x75, 0x75, 0x76, 0x77, 0x7A, 0x79, 0x7B, 0x7B, 0x80, 0x80, 0x80, 0x81, 0x81, 0x82, 0x81, 0x83, 0x82, 0x84, 0x80, 0x80, 0x80, 0x81, 0x82, 0x82, 0x83, 0x83, 0x85, 0x84, 0x86, 0x83, 0x84, 0x84, 0x86, 0x85, 0x84, 0x84, 0x83, 0x85, 0x85, 0x88, 0x88, 0x89, 0x88, 0x8A, 0x8A, 0x8B, 0x8D, 0x8D, 0x8E, 0x8C, 0x8D, 0x8B, 0x8C, 0x8C, 0x8E, 0x8D, 0x8F, 0x8F, 0x8F, 0x8F, 0x8D, 0x8E, 0x8B, 0x8C, 0x89, 0x88, 0x86, 0x85, 0x84, 0x82, 0x84, 0x84, 0x85, 0x82, 0x83, 0x82, 0x82, 0x83, 0x83, 0x84, 0x82, 0x84, 0x82, 0x83, 0x80, 0x7E, 0x7F, 0x7E, 0x7F, 0x7B, 0x7A, 0x78, 0x77, 0x75, 0x73, 0x74, 0x71, 0x70, 0x6F, 0x72, 0x73, 0x70, 0x70, 0x6D, 0x6C, 0x69, 0x6A, 0x6C, 0x6F, 0x72, 0x72, 0x75, 0x74, 0x76, 0x74, 0x75, 0x78, 0x79, 0x7B, 0x78, 0x77, 0x74, 0x75, 0x75, 0x75, 0x76, 0x75, 0x77, 0x77, 0x79, 0x79, 0x7A, 0x79, 0x7B, 0x80, 0x81, 0x83, 0x80, 0x81, 0x81, 0x81, 0x81, 0x82, 0x83, 0x82, 0x85, 0x85, 0x88, 0x89, 0x88, 0x88, 0x88, 0x8B, 0x8A, 0x8B, 0x8A, 0x8C, 0x8C, 0x8C, 0x8C, 0x8C, 0x91, 0x92, 0x94, 0x93, 0x96, 0x96, 0x96, 0x96, 0x95, 0x96, 0x94, 0x95, 0x91, 0x8F, 0x8B, 0x8A, 0x8B, 0x8F, 0x91, 0x90, 0x8F, 0x8C, 0x8E, 0x8D, 0x8F, 0x8E, 0x8E, 0x8E, 0x8C, 0x8B, 0x87, 0x85, 0x83, 0x84, 0x81, 0x80, 0x7D, 0x7C, 0x7B, 0x79, 0x77, 0x76, 0x79, 0x79, 0x78, 0x74, 0x73, 0x71, 0x6F, 0x6E, 0x6D, 0x6E, 0x6D, 0x6D, 0x6C, 0x70, 0x70, 0x72, 0x72, 0x74, 0x75, 0x74, 0x73, 0x6E, 0x6D, 0x6B, 0x6C, 0x6A, 0x6B, 0x6C, 0x70, 0x70, 0x6E, 0x6F, 0x71, 0x73, 0x72, 0x75, 0x77, 0x7B, 0x7B, 0x7B, 0x79, 0x79, 0x7B, 0x7B, 0x7B, 0x7A, 0x7D, 0x7D, 0x7E, 0x7F, 0x82, 0x85, 0x88, 0x8A, 0x89, 0x8B, 0x8C, 0x8E, 0x8E, 0x8F, 0x90, 0x8E, 0x8F, 0x8E, 0x92, 0x93, 0x95, 0x94, 0x96, 0x9B, 0x9C, 0x9D, 0x9C, 0x9F, 0x9C, 0x97, 0x95, 0x97, 0x9B, 0x9A, 0x9C, 0x9A, 0x9B, 0x98, 0x96, 0x96, 0x98, 0x9B, 0x96, 0x93, 0x8E, 0x8D, 0x8C, 0x8A, 0x8A, 0x89, 0x89, 0x85, 0x84, 0x81, 0x80, 0x80, 0x7E, 0x7E, 0x7B, 0x7B, 0x77, 0x76, 0x75, 0x73, 0x71, 0x6D, 0x70, 0x73, 0x75, 0x71, 0x6F, 0x6E, 0x6F, 0x70, 0x6D, 0x6C, 0x68, 0x68, 0x67, 0x69, 0x69, 0x65, 0x65, 0x64, 0x64, 0x5E, 0x5D, 0x62, 0x6C, 0x73, 0x72, 0x6F, 0x6D, 0x73, 0x75, 0x74, 0x6F, 0x6F, 0x70, 0x71, 0x71, 0x6F, 0x71, 0x74, 0x7D, 0x80, 0x80, 0x7C, 0x7E, 0x86, 0x8C, 0x8D, 0x87, 0x88, 0x8A, 0x8F, 0x8D, 0x8F, 0x94, 0x9B, 0xA4, 0xA4, 0x9D, 0x94, 0x97, 0x9C, 0xA1, 0x9D, 0x9C, 0x9F, 0xA4, 0xA0, 0x94, 0x92, 0x9F, 0xB6, 0xBC, 0xB5, 0xA5, 0x9A, 0x96, 0x9B, 0x9E, 0x98, 0x90, 0x8C, 0x8C, 0x83, 0x7E, 0x84, 0x95, 0x97, 0x8C, 0x7C, 0x72, 0x77, 0x81, 0x8C, 0x8C, 0x84, 0x77, 0x78, 0x7C, 0x7A, 0x72, 0x74, 0x7A, 0x74, 0x5F, 0x4E, 0x60, 0x78, 0x74, 0x5C, 0x56, 0x5B, 0x5D, 0x56, 0x50, 0x59, 0x72, 0x89, 0x81, 0x67, 0x50, 0x5A, 0x74, 0x74, 0x4D, 0x3C, 0x5A, 0x6E, 0x60, 0x59, 0x7B, 0x92, 0x7D, 0x52, 0x4A, 0x6C, 0x99, 0xB1, 0xA1, 0x81, 0x69, 0x78, 0x95, 0x86, 0x62, 0x73, 0xA4, 0xA5, 0x7A, 0x76, 0xAC, 0xCF, 0xA6, 0x6C, 0x73, 0xAB, 0xD4, 0xDA, 0xC5, 0x9A, 0x80, 0xA5, 0xC6, 0x8D, 0x5C, 0x94, 0xE1, 0xC4, 0x6B, 0x74, 0xC6, 0xC9, 0x70, 0x53, 0x93, 0xC8, 0xC2, 0xB3, 0xA2, 0x79, 0x78, 0xA6, 0x96, 0x4B, 0x48, 0xA7, 0xC0, 0x63, 0x3E, 0x85, 0xB7, 0x6D, 0x34, 0x63, 0xA9, 0x94, 0x7B, 0x83, 0x62, 0x35, 0x5E, 0x8E, 0x40, 0x10, 0x6D, 0xBB, 0x58, 0x0B, 0x55, 0xAB, 0x58, 0x09, 0x4D, 0xA4, 0x79, 0x58, 0x83, 0x61, 0x1D, 0x4B, 0x9C, 0x63, 0x26, 0x82, 0xC4, 0x69, 0x1E, 0x84, 0xBE, 0x62, 0x2C, 0x94, 0xC2, 0x86, 0x72, 0xAE, 0x82, 0x51, 0x8D, 0xC9, 0x7A, 0x56, 0xC0, 0xE2, 0x6D, 0x5B, 0xD8, 0xD1, 0x4E, 0x76, 0xFA, 0xD7, 0x94, 0xB2, 0xBD, 0x6E, 0x79, 0xC7, 0xAD, 0x58, 0x93, 0xE2, 0x9A, 0x58, 0xA9, 0xD8, 0x6E, 0x5F, 0xD9, 0xCB, 0x8D, 0x8D, 0x9F, 0x6E, 0x65, 0x93, 0x87, 0x4D, 0x69, 0xA4, 0x6A, 0x35, 0x6E, 0x8D, 0x32, 0x38, 0xA0, 0x98, 0x51, 0x60, 0x74, 0x40, 0x37, 0x68, 0x5B, 0x23, 0x52, 0x8B, 0x41, 0x1A, 0x5E, 0x71, 0x27, 0x4B, 0xA3, 0x89, 0x49, 0x69, 0x77, 0x52, 0x56, 0x7D, 0x70, 0x5E, 0x8E, 0x9F, 0x6D, 0x70, 0xA9, 0x90, 0x6B, 0xA4, 0xD8, 0xA2, 0x7E, 0xA2, 0xA3, 0x83, 0x97, 0xB7, 0x98, 0x95, 0xD2, 0xC0, 0x86, 0xA2, 0xC7, 0x8A, 0x8A, 0xE0, 0xE0, 0xA5, 0xA8, 0xBE, 0xA5, 0x92, 0xAD, 0xAA, 0x8A, 0x9E, 0xAC, 0x7E, 0x80, 0x9F, 0x7C, 0x6B, 0x9F, 0xA9, 0x7B, 0x6F, 0x7F, 0x76, 0x55, 0x60, 0x79, 0x55, 0x58, 0x7C, 0x59, 0x34, 0x5B, 0x56, 0x2B, 0x5D, 0x96, 0x67, 0x35, 0x43, 0x59, 0x3A, 0x34, 0x5F, 0x59, 0x41, 0x6F, 0x6E, 0x40, 0x5D, 0x74, 0x53, 0x71, 0xAE, 0x95, 0x63, 0x72, 0x8C, 0x7C, 0x72, 0x9A, 0x98, 0x79, 0x9F, 0xB4, 0x78, 0x8C, 0xB8, 0x85, 0x93, 0xE6, 0xDC, 0x93, 0x99, 0xB6, 0xAA, 0x9A, 0xC0, 0xC9, 0xA6, 0xC4, 0xD4, 0x9F, 0xA2, 0xC0, 0x90, 0x99, 0xD4, 0xBE, 0x96, 0x91, 0x9A, 0x90, 0x86, 0x8C, 0x8E, 0x76, 0x85, 0x8A, 0x5F, 0x61, 0x73, 0x4C, 0x5B, 0x9C, 0x81, 0x4D, 0x48, 0x5E, 0x52, 0x40, 0x50, 0x4A, 0x33, 0x58, 0x5F, 0x23, 0x34, 0x50, 0x38, 0x57, 0x96, 0x72, 0x36, 0x4C, 0x68, 0x61, 0x63, 0x78, 0x6B, 0x6E, 0x90, 0x86, 0x6D, 0x85, 0x82, 0x79, 0xB0, 0xCD, 0x96, 0x74, 0x99, 0xA0, 0x9D, 0xB5, 0xB4, 0x9A, 0xBD, 0xD5, 0xA8, 0xAA, 0xBB, 0x97, 0xAE, 0xF5, 0xD9, 0xA1, 0xAC, 0xB4, 0xA4, 0xB4, 0xAE, 0x90, 0x9F, 0xAA, 0x8C, 0x89, 0x92, 0x71, 0x73, 0xA2, 0x8E, 0x70, 0x6C, 0x60, 0x61, 0x77, 0x5F, 0x4A, 0x6C, 0x5B, 0x2E, 0x45, 0x54, 0x1B, 0x26, 0x79, 0x73, 0x31, 0x32, 0x49, 0x42, 0x3B, 0x52, 0x61, 0x50, 0x47, 0x61, 0x68, 0x5A, 0x6A, 0x8E, 0x91, 0x6E, 0x6C, 0x7E, 0x85, 0x87, 0x8E, 0x9D, 0xA7, 0xA6, 0x95, 0x8D, 0x9C, 0xBF, 0xDB, 0xDA, 0xC8, 0xBA, 0xC1, 0xCA, 0xCC, 0xC8, 0xC5, 0xC6, 0xC2, 0xB7, 0xA3, 0x98, 0x9D, 0xAE, 0xB0, 0xA4, 0x96, 0x8A, 0x87, 0x83, 0x7F, 0x77, 0x75, 0x77, 0x73, 0x63, 0x48, 0x40, 0x54, 0x6F, 0x69, 0x4D, 0x34, 0x2D, 0x35, 0x38, 0x3A, 0x3C, 0x45, 0x46, 0x3C, 0x30, 0x34, 0x4C, 0x65, 0x70, 0x64, 0x5B, 0x5F, 0x69, 0x6E, 0x74, 0x7B, 0x82, 0x88, 0x85, 0x84, 0x8C, 0xA7, 0xC2, 0xCA, 0xB9, 0xAD, 0xB3, 0xBB, 0xC4, 0xCA, 0xD1, 0xD1, 0xCC, 0xBD, 0xB6, 0xBC, 0xC8, 0xC8, 0xB9, 0xAB, 0xA2, 0xA0, 0x9A, 0x97, 0x91, 0x8D, 0x86, 0x78, 0x6B, 0x6A, 0x7A, 0x82, 0x78, 0x5E, 0x4D, 0x48, 0x48, 0x45, 0x45, 0x47, 0x3F, 0x30, 0x22, 0x2C, 0x43, 0x58, 0x51, 0x3E, 0x39, 0x44, 0x4F, 0x56, 0x5E, 0x63, 0x65, 0x62, 0x66, 0x70, 0x84, 0x95, 0x97, 0x8C, 0x8B, 0x96, 0xA0, 0xAC, 0xB3, 0xB5, 0xB3, 0xB6, 0xC1, 0xD3, 0xE7, 0xEA, 0xDB, 0xC5, 0xBD, 0xBB, 0xBD, 0xBA, 0xB3, 0xAA, 0x9F, 0x99, 0x93, 0x97, 0x98, 0x91, 0x7F, 0x70, 0x6F, 0x70, 0x6F, 0x64, 0x5B, 0x4F, 0x42, 0x3A, 0x49, 0x63, 0x5A, 0x35, 0x20, 0x2B, 0x38, 0x3A, 0x3A, 0x3C, 0x3E, 0x3D, 0x46, 0x59, 0x6F, 0x70, 0x61, 0x65, 0x73, 0x7E, 0x7E, 0x83, 0x8B, 0x90, 0x92, 0x9E, 0xBE, 0xD2, 0xC6, 0xB2, 0xBC, 0xCD, 0xD1, 0xD3, 0xD6, 0xD4, 0xC3, 0xBB, 0xC2, 0xCB, 0xBF, 0xAA, 0xA1, 0x9F, 0x9E, 0x96, 0x94, 0x8B, 0x78, 0x64, 0x6A, 0x86, 0x83, 0x6A, 0x57, 0x59, 0x54, 0x4D, 0x45, 0x34, 0x1B, 0x14, 0x36, 0x62, 0x5F, 0x33, 0x23, 0x30, 0x3F, 0x4C, 0x56, 0x58, 0x50, 0x5D, 0x79, 0x90, 0x83, 0x6E, 0x76, 0x8C, 0xA2, 0xAF, 0xB1, 0xA1, 0x9F, 0xBD, 0xEA, 0xED, 0xCC, 0xC0, 0xD7, 0xE4, 0xD8, 0xC8, 0xB0, 0xA3, 0xAB, 0xC0, 0xBA, 0xA5, 0x98, 0x96, 0x94, 0x88, 0x7A, 0x60, 0x55, 0x6F, 0x8F, 0x7C, 0x53, 0x39, 0x3C, 0x40, 0x2D, 0x1F, 0x22, 0x2D, 0x3A, 0x45, 0x37, 0x31, 0x3A, 0x47, 0x52, 0x55, 0x56, 0x5A, 0x60, 0x72, 0x92, 0x8D, 0x7D, 0x96, 0xA7, 0x98, 0x9A, 0xAC, 0xB0, 0xD8, 0xFF, 0xF8, 0xD0, 0xCF, 0xD1, 0xC9, 0xBD, 0xAC, 0xB8, 0xD8, 0xC7, 0xA3, 0x97, 0x9C, 0x94, 0x7A, 0x6E, 0x88, 0xA3, 0x84, 0x50, 0x49, 0x5F, 0x3C, 0x14, 0x2A, 0x69, 0x6F, 0x21, 0x00, 0x2D, 0x37, 0x1F, 0x33, 0x5A, 0x73, 0x55, 0x3E, 0x5D, 0x69, 0x5E, 0x6E, 0x93, 0xB0, 0x98, 0x7C, 0xA5, 0xB4, 0xA6, 0xC0, 0xF6, 0xFF, 0xD3, 0xBF, 0xD8, 0xCE, 0xB5, 0xC2, 0xD4, 0xC9, 0xAD, 0xA4, 0xA4, 0x8E, 0x86, 0x8E, 0x94, 0x83, 0x6C, 0x69, 0x5B, 0x4A, 0x4D, 0x61, 0x5C, 0x36, 0x27, 0x2B, 0x35, 0x3F, 0x52, 0x5A, 0x45, 0x38, 0x43, 0x55, 0x64, 0x7C, 0x88, 0x75, 0x64, 0x78, 0x93, 0xA7, 0xC8, 0xDD, 0xBF, 0x98, 0xAD, 0xCC, 0xD0, 0xDA, 0xE2, 0xC5, 0xA0, 0xA6, 0xB4, 0xAC, 0xAC, 0xAF, 0x96, 0x78, 0x7B, 0x82, 0x7B, 0x82, 0x7E, 0x51, 0x31, 0x3A, 0x40, 0x46, 0x5D, 0x49, 0x12, 0x14, 0x30, 0x3D, 0x54, 0x68, 0x49, 0x31, 0x4E, 0x62, 0x73, 0x93, 0x9B, 0x6D, 0x6E, 0x9C, 0xB3, 0xD0, 0xEA, 0xC3, 0x8C, 0xAE, 0xD7, 0xD7, 0xE3, 0xD6, 0xA1, 0x92, 0xB1, 0xB0, 0xAE, 0xB2, 0x92, 0x6C, 0x77, 0x82, 0x80, 0x8A, 0x68, 0x28, 0x1C, 0x42, 0x5D, 0x75, 0x4E, 0x0F, 0x10, 0x3A, 0x59, 0x78, 0x6C, 0x38, 0x3A, 0x5D, 0x82, 0xAE, 0xB3, 0x74, 0x55, 0x7B, 0xBA, 0xF8, 0xFF, 0xCE, 0x8E, 0x7F, 0xAB, 0xF0, 0xFC, 0xD9, 0xAE, 0x8C, 0x80, 0xA4, 0xC8, 0xB9, 0x9A, 0x7C, 0x5B, 0x47, 0x63, 0x80, 0x6E, 0x49, 0x2B, 0x18, 0x15, 0x2F, 0x44, 0x48, 0x45, 0x44, 0x44, 0x48, 0x57, 0x64, 0x72, 0x7C, 0x89, 0x90, 0x8E, 0x96, 0xAC, 0xCC, 0xDC, 0xDF, 0xDA, 0xD1, 0xC9, 0xC6, 0xCB, 0xC8, 0xC0, 0xB3, 0xAB, 0xA3, 0x99, 0x92, 0x88, 0x7D, 0x6D, 0x60, 0x51, 0x43, 0x39, 0x2F, 0x2A, 0x28, 0x2D, 0x2F, 0x33, 0x3C, 0x42, 0x48, 0x4F, 0x5D, 0x66, 0x73, 0x7F, 0x8A, 0x95, 0xA4, 0xB5, 0xC2, 0xCE, 0xD0, 0xCD, 0xCE, 0xD2, 0xCF, 0xC5, 0xC1, 0xB9, 0xB2, 0xA9, 0xA2, 0x98, 0x8D, 0x83, 0x74, 0x65, 0x54, 0x47, 0x38, 0x2E, 0x27, 0x23, 0x24, 0x27, 0x2E, 0x34, 0x3E, 0x45, 0x4F, 0x59, 0x64, 0x71, 0x7F, 0x8E, 0x9A, 0xA9, 0xB9, 0xCA, 0xD4, 0xDA, 0xD8, 0xD3, 0xCE, 0xC8, 0xC4, 0xBD, 0xB6, 0xAB, 0xA4, 0x9B, 0x92, 0x86, 0x76, 0x61, 0x4D, 0x3D, 0x2F, 0x28, 0x21, 0x1E, 0x1E, 0x25, 0x2D, 0x36, 0x40, 0x47, 0x52, 0x5C, 0x6A, 0x77, 0x8A, 0x99, 0xAB, 0xBC, 0xCB, 0xD7, 0xDE, 0xE0, 0xDA, 0xD6, 0xCE, 0xC8, 0xBF, 0xB8, 0xAE, 0xA5, 0x99, 0x8C, 0x7E, 0x6C, 0x59, 0x42, 0x2F, 0x20, 0x1C, 0x19, 0x1D, 0x23, 0x2B, 0x34, 0x3E, 0x49, 0x52, 0x5F, 0x69, 0x7A, 0x8D, 0xA1, 0xB2, 0xC5, 0xD5, 0xDF, 0xE4, 0xE1, 0xDD, 0xD3, 0xCC, 0xC5, 0xBE, 0xB4, 0xA8, 0x9A, 0x88, 0x78, 0x68, 0x58, 0x3F, 0x2C, 0x20, 0x1A, 0x1B, 0x21, 0x2A, 0x30, 0x3A, 0x45, 0x51, 0x5E, 0x6B, 0x7A, 0x88, 0x9E, 0xB3, 0xC5, 0xD2, 0xDC, 0xE0, 0xDD, 0xDD, 0xD6, 0xCE, 0xC3, 0xBB, 0xB1, 0xA2, 0x95, 0x84, 0x73, 0x61, 0x4D, 0x30, 0x1D, 0x1B, 0x1C, 0x24, 0x2D, 0x33, 0x39, 0x47, 0x52, 0x57, 0x65, 0x7C, 0x94, 0xA4, 0xB4, 0xBE, 0xC6, 0xDE, 0xF3, 0xEE, 0xD6, 0xC9, 0xC0, 0xBD, 0xBE, 0xB2, 0x99, 0x7D, 0x6A, 0x5E, 0x5E, 0x45, 0x1A, 0x0C, 0x16, 0x25, 0x36, 0x40, 0x3B, 0x43, 0x51, 0x60, 0x7B, 0x91, 0x93, 0x9A, 0xB5, 0xCF, 0xEE, 0xF3, 0xDD, 0xD2, 0xCE, 0xCA, 0xC4, 0xB8, 0xA0, 0x91, 0x84, 0x79, 0x6D, 0x4A, 0x25, 0x19, 0x1F, 0x2C, 0x36, 0x2F, 0x32, 0x41, 0x50, 0x66, 0x6F, 0x71, 0x7E, 0x98, 0xBB, 0xDD, 0xD7, 0xCB, 0xD2, 0xDA, 0xDE, 0xCF, 0xBB, 0xAD, 0xA6, 0x9E, 0x91, 0x6E, 0x4D, 0x3D, 0x39, 0x38, 0x26, 0x1A, 0x23, 0x36, 0x46, 0x51, 0x50, 0x5B, 0x72, 0x8F, 0xA8, 0xAE, 0xB8, 0xCF, 0xE4, 0xE9, 0xDD, 0xCB, 0xC5, 0xC0, 0xB6, 0xA6, 0x8C, 0x77, 0x66, 0x57, 0x3E, 0x22, 0x15, 0x1E, 0x2C, 0x3A, 0x3E, 0x41, 0x4F, 0x63, 0x7E, 0x8E, 0x96, 0xAA, 0xCE, 0xE9, 0xEC, 0xDB, 0xD1, 0xCF, 0xCB, 0xC0, 0xA6, 0x91, 0x84, 0x7C, 0x62, 0x35, 0x19, 0x19, 0x25, 0x2B, 0x2C, 0x31, 0x42, 0x56, 0x67, 0x72, 0x7E, 0x9D, 0xC1, 0xDF, 0xE2, 0xDD, 0xDB, 0xD7, 0xD2, 0xC1, 0xB0, 0x9E, 0x91, 0x7F, 0x5D, 0x38, 0x22, 0x1F, 0x25, 0x27, 0x26, 0x32, 0x43, 0x57, 0x67, 0x72, 0x8A, 0xAA, 0xCE, 0xDF, 0xDD, 0xDE, 0xDB, 0xD5, 0xC6, 0xB6, 0xA5, 0x97, 0x85, 0x69, 0x44, 0x28, 0x1E, 0x1F, 0x26, 0x28, 0x31, 0x41, 0x52, 0x68, 0x74, 0x89, 0xAA, 0xCE, 0xE3, 0xE3, 0xDC, 0xD8, 0xD2, 0xC4, 0xB4, 0x9F, 0x8E, 0x77, 0x5B, 0x3C, 0x1E, 0x16, 0x1D, 0x2C, 0x34, 0x3B, 0x49, 0x62, 0x7F, 0x96, 0xA8, 0xC2, 0xDF, 0xE7, 0xE2, 0xD0, 0xBB, 0xA3, 0x8E, 0x86, 0x7F, 0x77, 0x71, 0x6F, 0x6A, 0x68, 0x66, 0x65, 0x65, 0x64, 0x67, 0x66, 0x69, 0x69, 0x6D, 0x6E, 0x71, 0x72, 0x74, 0x77, 0x78, 0x7B, 0x7B, 0x7E, 0x7E, 0x81, 0x80, 0x82, 0x83, 0x84, 0x85, 0x85, 0x87, 0x86, 0x88, 0x87, 0x89, 0x88, 0x88, 0x88, 0x88, 0x87, 0x86, 0x87, 0x85, 0x86, 0x83, 0x84, 0x83, 0x84, 0x83, 0x83, 0x83, 0x83, 0x84, 0x82, 0x83, 0x82, 0x82, 0x80, 0x82, 0x80, 0x80, 0x7F, 0x7F, 0x7E, 0x7D, 0x7D, 0x7B, 0x7C, 0x7A, 0x7B, 0x7A, 0x7C, 0x7B, 0x7C, 0x7C, 0x7C, 0x7D, 0x7D, 0x7E, 0x7C, 0x7D, 0x7C, 0x7D, 0x7C, 0x7C, 0x7C, 0x7A, 0x7B, 0x7A, 0x7B, 0x7A, 0x7B, 0x7B, 0x7B, 0x7C, 0x7C, 0x7E, 0x7C, 0x7E, 0x7D, 0x7E, 0x7E, 0x7E, 0x7F, 0x7D, 0x7F, 0x7D, 0x7E, 0x7E, 0x7E, 0x80, 0x7E, 0x80, 0x7F, 0x81, 0x80, 0x80, 0x80, 0x7F, 0x80, 0x7E, 0x80, 0x7F, 0x80, 0x80, 0x7F, 0x80, 0x7F, 0x81, 0x80, 0x82, 0x82, 0x83, 0x84, 0x83, 0x84, 0x82, 0x83, 0x81, 0x82, 0x82, 0x82, 0x82, 0x81, 0x82, 0x80, 0x82, 0x81, 0x82, 0x82, 0x82, 0x83, 0x82, 0x83, 0x82, 0x83, 0x82, 0x83, 0x83, 0x82, 0x83, 0x81, 0x82, 0x81, 0x82, 0x81, 0x82, 0x82, 0x81, 0x82, 0x80, 0x81, 0x80, 0x81, 0x80, 0x81, 0x81, 0x80, 0x81, 0x80, 0x81, 0x80, 0x81, 0x80, 0x81, 0x81, 0x81, 0x81, 0x80, 0x81, 0x7F, 0x81, 0x7F, 0x80, 0x80, 0x81, 0x81, 0x80, 0x81, 0x7F, 0x80, 0x7E, 0x80, 0x7E, 0x80, 0x7F, 0x80, 0x80, 0x80, 0x80, 0x7F, 0x80, 0x7F, 0x80, 0x7F, 0x81, 0x80, 0x80, 0x80, 0x7F, 0x81, 0x7F, 0x80, 0x7E, 0x7F, 0x7E, 0x7F, 0x7F, 0x7F, 0x80, 0x7F, 0x80, 0x7F, 0x80, 0x7E, 0x80, 0x7E, 0x80, 0x7F, 0x80, 0x80, 0x7F, 0x80, 0x7E, 0x80, 0x7E, 0x80, 0x7F, 0x81, 0x80, 0x81, 0x80, 0x80, 0x80, 0x7F, 0x81, 0x80, 0x82, 0x80, 0x81, 0x80, 0x81, 0x81, 0x80, 0x81, 0x80, 0x81, 0x80, 0x82, 0x80, 0x81, 0x81, 0x81, 0x81, 0x80, 0x81, 0x80, 0x82, 0x81, 0x81, 0x81, 0x80, 0x82, 0x80, 0x82, 0x80, 0x81, 0x81, 0x81, 0x82, 0x81, 0x82, 0x80, 0x81, 0x80, 0x81, 0x81, 0x80, 0x82, 0x80, 0x82, 0x80, 0x81, 0x80, 0x81, 0x81, 0x80, 0x81, 0x80, 0x81, 0x80, 0x81, 0x81, 0x00]
    }

    fn gif_vec() -> Vec<u8> {
        vec![0x47, 0x49, 0x46, 0x38, 0x39, 0x61, 0x64, 0x00, 0x64, 0x00, 0xE6, 0x00, 0x00, 0x00, 0x18, 0x1E, 0xA5, 0xFA, 0xF4, 0x3D, 0x72, 0x5D, 0xF4, 0x23, 0x14, 0xA8, 0x0F, 0x09, 0xBA, 0x84, 0x4A, 0x6D, 0x0E, 0x10, 0x30, 0xA1, 0xEA, 0x1B, 0x39, 0x3F, 0xAB, 0xE2, 0xE7, 0x61, 0xA5, 0x82, 0x5C, 0x0E, 0x0C, 0xD2, 0xB9, 0xA6, 0xFF, 0xFF, 0xD9, 0x7F, 0x37, 0x1D, 0x47, 0x07, 0x0E, 0x94, 0x7D, 0x4A, 0xE1, 0x20, 0x12, 0x68, 0xC2, 0xD7, 0x49, 0x33, 0x47, 0xD3, 0x17, 0x08, 0x94, 0x9F, 0x76, 0x94, 0x0A, 0x07, 0xFE, 0x31, 0x18, 0x33, 0x00, 0x00, 0xE8, 0xF6, 0xCB, 0xF7, 0xEF, 0xDE, 0xBF, 0x27, 0x1C, 0xBA, 0x9D, 0x83, 0x28, 0x5C, 0x4E, 0x83, 0x08, 0x12, 0x3E, 0x72, 0x8D, 0x48, 0x3E, 0x2C, 0xFF, 0xFF, 0xFF, 0xF4, 0x27, 0x1A, 0x0B, 0x2B, 0x2F, 0x67, 0xA8, 0xBA, 0xB0, 0x21, 0x1B, 0xEF, 0xD6, 0xB8, 0x4A, 0x54, 0x4E, 0xDC, 0xE6, 0xD9, 0x5B, 0x29, 0x20, 0x97, 0x7C, 0x6D, 0x92, 0xA9, 0xB2, 0x71, 0x5D, 0x56, 0xEE, 0xFF, 0xFF, 0xD7, 0xFE, 0xE4, 0x32, 0x4A, 0x53, 0x43, 0xAA, 0xFD, 0x6E, 0x22, 0x19, 0xAD, 0xAA, 0x58, 0x24, 0x0F, 0x12, 0x2E, 0x35, 0x28, 0x86, 0x25, 0x22, 0xF8, 0xC5, 0x98, 0x26, 0x5C, 0x8A, 0x4C, 0x6D, 0x84, 0x9E, 0xEA, 0xE0, 0xFE, 0x32, 0x20, 0x65, 0x5E, 0x49, 0xC5, 0xCE, 0xBD, 0xD8, 0xE7, 0xE4, 0xFF, 0x28, 0x19, 0x5F, 0x75, 0x73, 0xC1, 0x1B, 0x10, 0x47, 0x1D, 0x1F, 0x5A, 0x1B, 0x22, 0x47, 0x93, 0xB0, 0x2B, 0x3B, 0x5E, 0xEF, 0xFE, 0xF4, 0xD4, 0x26, 0x17, 0x46, 0xAB, 0xF7, 0xC5, 0xF0, 0xE6, 0x98, 0x22, 0x1B, 0xB5, 0x35, 0x21, 0xFF, 0xFF, 0xF1, 0x33, 0x66, 0x66, 0xCC, 0xFF, 0xFF, 0xB8, 0xB0, 0x8C, 0x7D, 0x8A, 0x8A, 0x7B, 0x50, 0x3D, 0xE0, 0x2A, 0x1B, 0xD0, 0xA4, 0x7C, 0x32, 0x1E, 0x21, 0x84, 0x71, 0x5D, 0x42, 0x49, 0x51, 0x87, 0x17, 0x19, 0xC0, 0x20, 0x19, 0x1A, 0x18, 0x11, 0x78, 0xD8, 0xC7, 0x4A, 0x61, 0x82, 0x59, 0x1B, 0x1B, 0x5F, 0x44, 0x2D, 0x70, 0x29, 0x31, 0x2D, 0x3A, 0x24, 0x48, 0xA5, 0xE9, 0xFF, 0x29, 0x10, 0xEA, 0xE4, 0xA9, 0x92, 0x91, 0x8C, 0xDC, 0xFF, 0xFF, 0xC0, 0xD6, 0xA2, 0x94, 0x64, 0x53, 0x6E, 0x18, 0x1D, 0x25, 0x1C, 0x20, 0xA4, 0x85, 0x64, 0xB2, 0x07, 0x0A, 0x45, 0xB9, 0xFF, 0x24, 0x46, 0x70, 0xC1, 0xF7, 0xF7, 0xFF, 0x3B, 0x24, 0xAB, 0x64, 0x37, 0x37, 0x1A, 0x1A, 0x37, 0x8F, 0x86, 0x38, 0x75, 0x6F, 0x08, 0x54, 0x76, 0xFF, 0xFF, 0xE8, 0xF1, 0xFF, 0xE4, 0x42, 0x5B, 0x6F, 0x8C, 0xC5, 0xB5, 0x7D, 0x50, 0x2B, 0xA9, 0x1B, 0x19, 0x8C, 0xCA, 0xD6, 0x66, 0x83, 0x86, 0x43, 0x13, 0x1A, 0xC8, 0xF1, 0xC0, 0x35, 0x31, 0x41, 0x8C, 0xE6, 0xFB, 0x11, 0x20, 0x29, 0x21, 0xF9, 0x04, 0x04, 0x14, 0x00, 0xFF, 0x00, 0x2C, 0x00, 0x00, 0x00, 0x00, 0x64, 0x00, 0x64, 0x00, 0x00, 0x07, 0xFF, 0x80, 0x21, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89, 0x8A, 0x8B, 0x8C, 0x8D, 0x8E, 0x8F, 0x90, 0x91, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9A, 0x9B, 0x9C, 0x9D, 0x9E, 0x9F, 0xA0, 0xA1, 0xA2, 0xA3, 0xA4, 0xA5, 0xA6, 0xA7, 0xA8, 0xA9, 0xAA, 0xAB, 0xAC, 0xAD, 0xAE, 0xAF, 0xB0, 0xB1, 0xB2, 0xB3, 0xB4, 0xB5, 0xB6, 0xB7, 0xB8, 0xB9, 0xBA, 0xBB, 0xBC, 0xBD, 0xBE, 0xBF, 0xC0, 0xC1, 0xC2, 0xC3, 0xC4, 0xC5, 0xA0, 0x73, 0x73, 0xC6, 0x99, 0x0D, 0x36, 0x05, 0x6E, 0x05, 0x52, 0x26, 0xCA, 0x95, 0x26, 0x05, 0x3B, 0x6B, 0x37, 0x37, 0x55, 0x5C, 0x52, 0xD3, 0x92, 0x19, 0xD7, 0x07, 0x07, 0x30, 0xE4, 0x72, 0x7B, 0x0C, 0xDE, 0x90, 0x32, 0x08, 0x07, 0x6A, 0xEE, 0xEE, 0x07, 0x23, 0x68, 0xE9, 0x8E, 0x74, 0x20, 0x6B, 0x6A, 0x30, 0x47, 0xFB, 0x47, 0x6A, 0x08, 0x29, 0xF4, 0x1A, 0xB9, 0xF0, 0x82, 0x8F, 0x1C, 0xBF, 0x23, 0x08, 0xB6, 0x2C, 0x09, 0xB8, 0x68, 0x8C, 0x17, 0x22, 0xF9, 0x0C, 0xEE, 0x1B, 0xB1, 0x83, 0x61, 0x22, 0x0D, 0x82, 0x40, 0x10, 0x21, 0xC7, 0x51, 0xCD, 0x97, 0x3F, 0x62, 0x04, 0x99, 0xC0, 0x68, 0x51, 0x10, 0x0A, 0x31, 0x1C, 0xC2, 0xEC, 0x20, 0xF2, 0xE5, 0x9D, 0xBB, 0x0F, 0x67, 0x9C, 0x64, 0x40, 0x03, 0x05, 0x45, 0x49, 0x41, 0x2D, 0x14, 0xFC, 0x01, 0x71, 0xE6, 0xCF, 0x8B, 0x3A, 0x1F, 0x3E, 0xC4, 0xA9, 0x83, 0x00, 0x00, 0x08, 0x10, 0x58, 0x42, 0x1A, 0x5A, 0x92, 0x0C, 0x58, 0x03, 0x13, 0x23, 0x09, 0xCD, 0xA9, 0x40, 0xC3, 0x4C, 0x0D, 0x33, 0x58, 0x85, 0x08, 0xD9, 0x12, 0xA3, 0x6B, 0x8D, 0x29, 0x7F, 0x2A, 0x34, 0x28, 0x94, 0x41, 0x06, 0x04, 0x29, 0x0D, 0xE6, 0x64, 0x88, 0x9A, 0x6B, 0x0E, 0x84, 0xAE, 0x65, 0xFF, 0x48, 0x66, 0x80, 0xE0, 0x60, 0x83, 0x8E, 0xBB, 0x77, 0xA3, 0xE8, 0xC5, 0xAB, 0x63, 0x43, 0x8D, 0x14, 0x3B, 0x38, 0x34, 0x2D, 0x42, 0x65, 0x81, 0x01, 0x21, 0x3B, 0x20, 0x70, 0x89, 0x11, 0xB7, 0x96, 0x5A, 0x0D, 0x4B, 0xC8, 0xA4, 0x48, 0x43, 0x00, 0x83, 0x93, 0x10, 0x19, 0x7E, 0x2C, 0x30, 0x72, 0xA1, 0xB3, 0xE7, 0xCF, 0x9D, 0x45, 0x6C, 0xF0, 0x61, 0xC4, 0xCC, 0x1E, 0x34, 0x63, 0x5D, 0x80, 0xA0, 0xA0, 0xA3, 0x06, 0x80, 0x18, 0x40, 0x08, 0x3C, 0x40, 0x37, 0x6B, 0xAE, 0x03, 0x28, 0x2C, 0x3A, 0x24, 0xB9, 0x8B, 0x41, 0x45, 0x03, 0x2A, 0x66, 0x22, 0xF8, 0xD0, 0xEB, 0xA3, 0xB8, 0x71, 0x1F, 0x57, 0x44, 0x0C, 0x8F, 0x21, 0xA2, 0x73, 0x12, 0x21, 0x54, 0x34, 0x8C, 0x11, 0x10, 0xA1, 0x6F, 0x8A, 0x28, 0x3A, 0xA2, 0xBC, 0xA1, 0x2D, 0xCB, 0x8E, 0x01, 0x0A, 0x1B, 0x62, 0x08, 0x89, 0x72, 0x41, 0x47, 0x97, 0x27, 0x15, 0xB6, 0x44, 0x70, 0x3E, 0x9A, 0x74, 0x89, 0xE2, 0x46, 0xCE, 0xBC, 0x17, 0x51, 0xA2, 0x79, 0xE7, 0x0D, 0x0F, 0xA8, 0x64, 0x50, 0x90, 0xE4, 0x82, 0x08, 0xCE, 0x17, 0xC4, 0xF0, 0x87, 0x13, 0x0B, 0x75, 0xC7, 0x1C, 0x68, 0x17, 0x40, 0xF1, 0x03, 0x08, 0x40, 0x78, 0xB6, 0x41, 0x7B, 0x35, 0x9C, 0xA1, 0x9C, 0x11, 0x53, 0x94, 0x80, 0x60, 0x67, 0x40, 0xEC, 0xB1, 0x83, 0x00, 0x35, 0x1C, 0x27, 0x42, 0x0D, 0x42, 0x74, 0xC1, 0xC1, 0x2C, 0x76, 0x8C, 0x50, 0x03, 0x68, 0x22, 0x74, 0x31, 0x42, 0x0C, 0xC7, 0xE9, 0xA5, 0x5C, 0x12, 0x41, 0x88, 0x00, 0x06, 0x18, 0x46, 0xD8, 0x77, 0x61, 0x09, 0x41, 0xD4, 0x10, 0xC5, 0x71, 0xC6, 0x79, 0xB0, 0x43, 0x81, 0xAF, 0x38, 0x71, 0x47, 0x0C, 0x4A, 0x20, 0x18, 0x03, 0x16, 0x57, 0xCC, 0x68, 0xDC, 0xFF, 0x8E, 0x3E, 0x0C, 0x60, 0xC4, 0x8C, 0x50, 0x42, 0xA9, 0x9C, 0x0F, 0x51, 0x46, 0x21, 0x82, 0x8C, 0x51, 0x92, 0xF6, 0x80, 0x0A, 0xB1, 0x20, 0xE1, 0x45, 0x1A, 0xA0, 0x19, 0xD7, 0x95, 0x94, 0x7E, 0x19, 0x31, 0x40, 0x94, 0x68, 0x82, 0x11, 0xC5, 0x06, 0x03, 0xB4, 0x99, 0x26, 0x9A, 0x46, 0x6C, 0x31, 0x02, 0x12, 0x5D, 0x76, 0x90, 0x06, 0x8F, 0xC6, 0xBD, 0x91, 0x04, 0x94, 0x03, 0x1C, 0x99, 0xE4, 0x9B, 0x50, 0x46, 0xB1, 0xC5, 0x16, 0x49, 0x3C, 0x09, 0xA8, 0x67, 0x78, 0x4C, 0x40, 0x87, 0x2C, 0x59, 0x54, 0xB1, 0xC5, 0x06, 0x36, 0x5E, 0x10, 0xC5, 0x19, 0x7F, 0xCE, 0x68, 0xC4, 0x1B, 0x25, 0x00, 0x0A, 0xA5, 0x0F, 0x41, 0x04, 0x71, 0xC5, 0x99, 0x70, 0x5A, 0x91, 0x44, 0x04, 0x60, 0x5C, 0x10, 0xC1, 0x1E, 0x3C, 0xC8, 0x32, 0x06, 0x1C, 0x6F, 0x90, 0x27, 0x29, 0x1E, 0x57, 0x58, 0x11, 0x23, 0x8F, 0x63, 0x6A, 0x7A, 0x41, 0x7C, 0x56, 0xBC, 0x09, 0xC4, 0x04, 0x1F, 0x0C, 0x71, 0x02, 0x05, 0x03, 0xA4, 0xD1, 0x87, 0x4D, 0xB1, 0x2C, 0x21, 0x00, 0x01, 0x78, 0x19, 0x40, 0x44, 0x1D, 0x7F, 0x74, 0xC8, 0x23, 0x8E, 0xA4, 0x6A, 0x9A, 0x04, 0x00, 0x7B, 0xA6, 0x69, 0xC0, 0x0F, 0x2D, 0x8C, 0x21, 0x01, 0x0B, 0x5D, 0x4C, 0xB0, 0xC2, 0x2C, 0x45, 0xC4, 0x41, 0x40, 0x67, 0x04, 0xF4, 0x91, 0x47, 0x00, 0xBA, 0x21, 0x18, 0x85, 0x10, 0xD5, 0xA6, 0xD9, 0x99, 0x19, 0x00, 0x54, 0xBA, 0xE9, 0x02, 0x2C, 0xE0, 0xC4, 0x46, 0x02, 0x09, 0xB4, 0x40, 0x0B, 0x09, 0x66, 0xA4, 0x61, 0x41, 0x1F, 0xDF, 0x86, 0x20, 0xEE, 0x85, 0x49, 0xBC, 0x01, 0x04, 0xA0, 0x22, 0x4C, 0x71, 0xC6, 0x00, 0xC6, 0xCD, 0x0B, 0x42, 0x53, 0xB8, 0xB0, 0x31, 0x84, 0x16, 0x7A, 0xE4, 0xA1, 0xAF, 0xFF, 0xC0, 0x1E, 0xE0, 0xD9, 0x64, 0xA7, 0xA0, 0xA2, 0x79, 0x05, 0x00, 0x5B, 0x14, 0x67, 0x2D, 0x0D, 0x24, 0xE5, 0xD2, 0x44, 0x13, 0x17, 0x0B, 0x02, 0x07, 0x06, 0x80, 0x5E, 0xA1, 0x67, 0x9A, 0x03, 0x6C, 0x01, 0xC0, 0x7B, 0x69, 0xEA, 0x60, 0xC5, 0x04, 0x40, 0xFE, 0xA2, 0x00, 0x16, 0x9A, 0x5E, 0xB1, 0x85, 0x19, 0x86, 0x46, 0x80, 0xA3, 0x9E, 0x3C, 0x7E, 0x66, 0x40, 0xBD, 0xC2, 0xD8, 0x01, 0x40, 0x83, 0x2D, 0x0F, 0x6A, 0xC6, 0x16, 0x6F, 0x9C, 0xB1, 0x85, 0x11, 0x78, 0x7A, 0x26, 0xC2, 0x03, 0x4A, 0x05, 0xD3, 0x84, 0x1C, 0x06, 0xF0, 0x88, 0xA6, 0xD0, 0x56, 0x74, 0x55, 0xDF, 0x85, 0x9E, 0x95, 0xDB, 0xC3, 0x2F, 0x28, 0x30, 0xE0, 0x84, 0x34, 0x59, 0x9C, 0x41, 0x81, 0x92, 0x1A, 0x17, 0x4D, 0xF6, 0x05, 0x03, 0x2C, 0x90, 0x35, 0x2F, 0x64, 0xEC, 0x10, 0xC3, 0x16, 0x29, 0x70, 0x80, 0x44, 0x1C, 0x7B, 0x74, 0x0C, 0x06, 0x5F, 0x71, 0x1F, 0xF7, 0x99, 0x05, 0x13, 0x10, 0xCB, 0x0B, 0x1F, 0x3B, 0xE0, 0x21, 0x42, 0x14, 0x78, 0x58, 0xA1, 0x82, 0x02, 0x23, 0x5C, 0x71, 0x9C, 0x0E, 0x6D, 0x64, 0xAE, 0xB1, 0xBB, 0x17, 0x00, 0x11, 0x44, 0xC0, 0xBD, 0x28, 0x60, 0x40, 0xE6, 0x6D, 0xE8, 0x60, 0x44, 0x17, 0x34, 0xC4, 0x90, 0x1C, 0x96, 0x17, 0x90, 0xEE, 0x99, 0xA6, 0xA5, 0x02, 0xF1, 0xC0, 0x13, 0x29, 0xEF, 0x52, 0x84, 0x00, 0x78, 0x64, 0x8E, 0x57, 0x09, 0x49, 0x5C, 0xF9, 0x38, 0x93, 0x3E, 0xE8, 0xF0, 0x19, 0xEC, 0x14, 0xA4, 0xA0, 0x47, 0x11, 0xBF, 0xB8, 0xC0, 0x04, 0x10, 0xA5, 0xF3, 0xA5, 0x83, 0x71, 0x1B, 0x08, 0x11, 0x43, 0x1A, 0x1D, 0xBF, 0x0E, 0x33, 0x01, 0x41, 0xD0, 0x0E, 0x4C, 0x0B, 0x70, 0x78, 0xE0, 0xBC, 0xF3, 0x35, 0xE0, 0x30, 0x84, 0xFF, 0x36, 0x31, 0x58, 0x40, 0x00, 0x01, 0x40, 0x08, 0xC7, 0xE3, 0x15, 0x42, 0xBC, 0x40, 0x42, 0xED, 0xBE, 0x48, 0x30, 0x45, 0x75, 0x7C, 0x91, 0x8E, 0x47, 0x15, 0x2E, 0xB4, 0xC0, 0x86, 0x1F, 0x24, 0x7C, 0x50, 0x07, 0x11, 0x44, 0x00, 0x01, 0x80, 0x42, 0x13, 0x84, 0x17, 0xB0, 0x81, 0x18, 0x4D, 0x80, 0x43, 0x91, 0xEA, 0x97, 0xB9, 0x07, 0xFC, 0xA0, 0x10, 0x4D, 0x60, 0xC3, 0xFE, 0x86, 0xD0, 0x2E, 0x30, 0xF8, 0x60, 0x0B, 0x7D, 0x60, 0x00, 0xC4, 0x82, 0x91, 0x80, 0x1F, 0x10, 0xA0, 0x54, 0xE5, 0xC1, 0x0B, 0x06, 0xEE, 0x56, 0x88, 0x1C, 0xF4, 0xE1, 0x60, 0x4A, 0xDA, 0x80, 0x19, 0x52, 0x80, 0x86, 0x92, 0x05, 0x23, 0x0F, 0x71, 0xB0, 0x02, 0xA9, 0xF0, 0x12, 0x81, 0x19, 0xA4, 0xCA, 0x10, 0x0D, 0xC8, 0x80, 0x13, 0x46, 0x80, 0x07, 0x1E, 0x89, 0xC0, 0x0A, 0x33, 0x18, 0x11, 0x31, 0x12, 0xF0, 0x81, 0x09, 0x10, 0x20, 0x02, 0x03, 0xA0, 0x80, 0x01, 0xF4, 0x00, 0xBF, 0x10, 0x84, 0x41, 0x06, 0x77, 0x50, 0x42, 0x09, 0x80, 0x57, 0x9C, 0xBB, 0x6C, 0xC9, 0x18, 0x4D, 0x90, 0x00, 0x0E, 0x4E, 0x70, 0x82, 0x2A, 0x3C, 0x61, 0x0C, 0x86, 0x20, 0xC3, 0x1D, 0x92, 0x40, 0x01, 0x1F, 0xCC, 0x0D, 0x6B, 0xD3, 0x60, 0x43, 0x0E, 0x72, 0x90, 0x00, 0xE4, 0x15, 0x82, 0x0E, 0x02, 0xF0, 0x80, 0x08, 0xEE, 0x42, 0x36, 0x02, 0xFC, 0x41, 0x71, 0x37, 0x71, 0x41, 0x15, 0x28, 0xD0, 0xBC, 0x0B, 0x45, 0xE0, 0x01, 0x7A, 0xB8, 0x09, 0x21, 0x90, 0xC0, 0x04, 0x0A, 0xCC, 0x8D, 0x02, 0x5B, 0xF8, 0x81, 0x1B, 0x05, 0x19, 0x82, 0x16, 0x90, 0x60, 0x02, 0x6F, 0x43, 0x50, 0x1A, 0x82, 0xA0, 0x07, 0x17, 0x30, 0x92, 0x10, 0x59, 0xAC, 0x83, 0x19, 0x08, 0x40, 0x81, 0x4E, 0x12, 0xC0, 0x46, 0x0C, 0x44, 0x20, 0x01, 0x18, 0x2F, 0x49, 0x88, 0x16, 0x24, 0xA0, 0x7F, 0x75, 0xA8, 0x42, 0x15, 0x70, 0xB0, 0x02, 0x24, 0x34, 0x91, 0x94, 0x8D, 0x6C, 0x42, 0x00, 0x72, 0x80, 0x84, 0x51, 0xC2, 0xF2, 0x96, 0xB8, 0xCC, 0xA5, 0x2E, 0x77, 0xC9, 0xCB, 0x5E, 0xFA, 0xF2, 0x97, 0xC0, 0x0C, 0xA6, 0x30, 0x87, 0x49, 0xCC, 0x62, 0x1A, 0xF3, 0x98, 0xC8, 0x4C, 0xA6, 0x32, 0x97, 0xC9, 0xCC, 0x66, 0x3A, 0xF3, 0x99, 0x8A, 0x08, 0x04, 0x00, 0x3B]
    }

    fn write_data_into_file<P: AsRef<Path>>(file_path: P, data: &[u8]) -> File {
        let mut tmpfile = File::create(file_path).unwrap();
        tmpfile.write_all(data).unwrap();
        tmpfile
    }

    #[test]
    pub fn test_fill_wav() {
        let file_path = "test_fill_wav.anyext";
        write_data_into_file(file_path, &wav_vec());

        // check that the samples are valid
        let at_least_one_not_zero = {
            let mut reader = WavReader::open(file_path).unwrap();
            reader.samples::<i16>().any(|s| s.unwrap() != 0)
        };

        fill_wav(file_path).unwrap();

        // check that the file does have all samples filled with zeroes
        let is_all_zeroes = {
            let mut reader = WavReader::open(file_path).unwrap();
            reader.samples::<i16>().all(|s| s.unwrap() == 0)
        };

        fs::remove_file(file_path).unwrap();

        assert!(at_least_one_not_zero);
        assert!(is_all_zeroes);
    }

    #[test]
    pub fn test_fill_any() {
        let file_path = "test_fill_any.gif";
        write_data_into_file(file_path, &gif_vec());

        // check that the data is valid
        let at_least_one_not_zero = {
            let mut buf = Vec::new();
            File::open(file_path).unwrap().read_to_end(&mut buf).unwrap();
            buf.into_iter().any(|d| d != 0)
        };

        fill_any(file_path).unwrap();

        // check that the file does have all samples filled with zeroes
        let is_all_zeroes = {
            let mut buf = Vec::new();
            File::open(file_path).unwrap().read_to_end(&mut buf).unwrap();
            buf.into_iter().all(|d| d == 0)
        };

        fs::remove_file(file_path).unwrap();

        assert!(at_least_one_not_zero);
        assert!(is_all_zeroes);
    }

    #[test]
    pub fn test_c_zero_fill_matching() {
        let file_path = "test_c_zero_fill_matching.WAV";
        write_data_into_file(file_path, &wav_vec());

        // check that the samples are valid
        let at_least_one_not_zero = {
            let mut reader = WavReader::open(file_path).unwrap();
            reader.samples::<i16>().any(|s| s.unwrap() != 0)
        };

        let file_path_with_nul = &format!("{}\0", file_path);
        let c_file_path = CStr::from_bytes_with_nul(file_path_with_nul.as_bytes()).unwrap();
        let status = zero_fill_matching(c_file_path.as_ptr());

        assert!(status == ZERO_FILL_OK);

        // check that the file does have all samples filled with zeroes
        let is_all_zeroes = {
            let mut reader = WavReader::open(file_path).unwrap();
            reader.samples::<i16>().all(|s| s.unwrap() == 0)
        };

        fs::remove_file(file_path).unwrap();

        assert!(at_least_one_not_zero);
        assert!(is_all_zeroes);
    }
}