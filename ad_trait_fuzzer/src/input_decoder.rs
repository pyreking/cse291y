// src/input_decoder.rs

use core::convert::TryInto;
use std::error::Error;

/// Defines the interface for converting raw fuzzer bytes into numerical inputs (f64).
pub trait FuzzInputDecoder {
    /// The exact number of f64 inputs this decoder expects to produce.
    fn num_inputs(&self) -> usize;
    
    /// The minimum number of bytes required to satisfy the input needs (NUM_INPUTS * 8).
    fn min_bytes(&self) -> usize { self.num_inputs() * 8 }
    
    /// Attempts to read and decode f64 inputs from the raw byte slice.
    fn decode(&self, data: &[u8]) -> Result<Vec<f64>, Box<dyn Error>>;
}

/// A concrete decoder for functions that require exactly two f64 inputs (x and y).
pub struct TwoInputDecoder;

impl FuzzInputDecoder for TwoInputDecoder {
    fn num_inputs(&self) -> usize { 2 }

    fn decode(&self, data: &[u8]) -> Result<Vec<f64>, Box<dyn Error>> {
        if data.len() < self.min_bytes() {
            return Err("Not enough data to decode inputs".into());
        }
        println!("data length: {}", data.len());
        // Decode x
        let x_bytes: [u8; 8] = data[0..8].try_into().map_err(|_| "Failed to slice x bytes")?;
        let x = f64::from_le_bytes(x_bytes);

        // Decode y
        let y_bytes: [u8; 8] = data[8..16].try_into().map_err(|_| "Failed to slice y bytes")?;
        let y = f64::from_le_bytes(y_bytes);

        Ok(vec![x, y])
    }
}

pub struct GeneralInputDecoder
{
    pub input_length: usize
}

impl FuzzInputDecoder for GeneralInputDecoder
{
    fn num_inputs(&self) -> usize { self.input_length }

    fn decode(&self, data: &[u8]) -> Result<Vec<f64>, Box<dyn Error>>
    {
        let mut ret_val: Vec<f64> = vec![];
        ret_val.resize(self.input_length, 0.0);
        for (i, el) in ret_val.iter_mut().enumerate()
        {
            let bytes: [u8; 8] = data[i..(8 + i)].try_into().map_err(|_| "Failed to slice bytes")?;
            *el = f64::from_le_bytes(bytes);
        } 
        println!("First vector value after decode: {}", ret_val[0]);
        return Ok(ret_val);
    }
}
