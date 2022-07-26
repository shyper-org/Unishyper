use core::slice;

use crate::drivers::blk;

use super::interface::{BlkIO, AtaError};

use lru::LruCache;

// reference: https://github.com/rafalh/rust-fatfs/issues/55
// https://github.com/x37v/stm32h7xx-hal/blob/xnor/fatfs/src/sdmmc.rs#L1392-L1697

pub const BSIZE: usize = 512;

#[derive(Debug)]
pub enum DiskCursorIoError {
    UnexpectedEof,
    WriteZero,
}

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
    fn read(&self, sector: usize, count: usize) -> Result<(), AtaError> {
        assert!(count == 1);
        blk::read(sector, count, self.0.as_ptr() as usize);
        Ok(())
    }

    fn write(&self, sector: usize, count: usize) -> Result<(), AtaError> {
        assert!(count == 1);
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

const MAX_LRU: usize = 16;

pub struct BlockCache {
    cache: LruCache<usize, DataBlock>,
}

impl BlockCache {
    pub fn new() -> Self {
        BlockCache {
            cache: LruCache::new(MAX_LRU),
        }
    }

    fn sector_cached(&self, sector: usize) -> bool {
        self.cache.contains(&sector)
    }

    pub fn get(&mut self, sector: usize, count: usize) -> &mut DataBlock {
        if !self.sector_cached(sector) {
            // Uncached
            let block = if self.cache.len() >= MAX_LRU {
                // LRU cache is full
                match self.cache.pop_lru() {
                    Some((_, block)) => block,
                    None => panic!("LRU Cache pop_lru error"),
                }
            } else {
                // not full
                DataBlock::new()
            };
            block.read(sector, count).expect("ata error");
            self.cache.push(sector, block);
            // peek it (without update)
            match self.cache.peek_mut(&sector) {
                Some(block) => block,
                None => panic!("LRU Cache peek_mut error"),
            }
        } else {
            match self.cache.get_mut(&sector) {
                Some(block) => block,
                None => panic!("LRU Cache get_mut error"),
            }
        }
    }
}

pub struct DiskCursor {
    pub sector: usize,
    pub offset: usize,
    // Block Cache
    pub cache: BlockCache,
}

impl DiskCursor {
    pub fn new(start_sector: usize) -> Self {
        DiskCursor {
            sector: start_sector,
            offset: 0,
            cache: BlockCache::new(),
        }
    }

    pub fn get_position(&self) -> usize {
        self.sector * BSIZE + self.offset
    }

    pub fn set_position(&mut self, position: usize) {
        self.sector = position / BSIZE;
        self.offset = position % BSIZE;
    }

    pub fn move_cursor(&mut self, amount: usize) {
        self.set_position(self.get_position() + amount)
    }
}
