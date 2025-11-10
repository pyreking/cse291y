// src/input_decoder.rs

use core::convert::TryInto;
use std::error::Error;

/// Defines the interface for converting raw fuzzer bytes into numerical inputs (f64).
pub trait FuzzInputDecoder {
    /// The exact number of f64 inputs this decoder expects to produce.
    const NUM_INPUTS: usize;
    
    /// The minimum number of bytes required to satisfy the input needs (NUM_INPUTS * 8).
    const MIN_BYTES: usize = Self::NUM_INPUTS * 8;
    
    /// Attempts to read and decode f64 inputs from the raw byte slice.
    fn decode(data: &[u8]) -> Result<Vec<f64>, Box<dyn Error>>;
}

/// A concrete decoder for functions that require exactly two f64 inputs (x and y).
pub struct TwoInputDecoder;

impl FuzzInputDecoder for TwoInputDecoder {
    const NUM_INPUTS: usize = 2;

    fn decode(data: &[u8]) -> Result<Vec<f64>, Box<dyn Error>> {
        if data.len() < Self::MIN_BYTES {
            return Err("Not enough data to decode inputs".into());
        }

        // Decode x
        let x_bytes: [u8; 8] = data[0..8].try_into().map_err(|_| "Failed to slice x bytes")?;
        let x = f64::from_le_bytes(x_bytes);

        // Decode y
        let y_bytes: [u8; 8] = data[8..16].try_into().map_err(|_| "Failed to slice y bytes")?;
        let y = f64::from_le_bytes(y_bytes);

        Ok(vec![x, y])
    }
}