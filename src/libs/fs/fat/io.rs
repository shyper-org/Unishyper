use core::slice;

use crate::drivers::blk;

use crate::libs::fs::interface::{BlkIO, AtaError};
use crate::libs::fs::fat::diskcursor::BSIZE;

// reference: https://github.com/rafalh/rust-fatfs/issues/55
// https://github.com/x37v/stm32h7xx-hal/blob/xnor/fatfs/src/sdmmc.rs#L1392-L1697
#[derive(Debug, Clone)]
#[repr(align(512))]
#[repr(C)]
pub struct DataBlock(pub [u8; BSIZE]);

#[allow(dead_code)]
impl DataBlock {
    pub const fn new() -> Self {
        Self([0; BSIZE])
    }

    pub fn blocks_to_words(blocks: &[DataBlock]) -> &[u32] {
        let word_count = blocks.len() * 128;
        // SAFETY: `DataBlock` is 4-byte aligned.
        unsafe { slice::from_raw_parts(blocks.as_ptr() as *mut u32, word_count) }
    }

    pub fn blocks_to_words_mut(blocks: &mut [DataBlock]) -> &mut [u32] {
        let word_count = blocks.len() * 128;
        // SAFETY: `DataBlock` is 4-byte aligned.
        unsafe { slice::from_raw_parts_mut(blocks.as_mut_ptr() as *mut u32, word_count) }
    }
}

impl BlkIO for DataBlock {
    fn read(&mut self, sector: usize, count: usize) -> Result<(), AtaError> {
        debug_assert!(count == 1);
        blk::read(sector, count, self.0.as_ptr() as usize);
        Ok(())
    }

    fn write(&self, sector: usize, count: usize) -> Result<(), AtaError> {
        debug_assert!(count == 1);
        blk::write(sector, count, self.0.as_ptr() as usize);
        Ok(())
    }

    fn get_data(&self, offset: usize) -> &[u8] {
        &self.0[offset..]
    }

    fn get_data_mut(&mut self, offset: usize) -> &mut [u8] {
        &mut self.0[offset..]
    }
}
