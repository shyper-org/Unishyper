pub mod fatfs2 {
    use fatfs::{IoBase, IoError, Read, Seek, SeekFrom, Write, FsOptions, FileSystem};
    use core::slice;

    use crate::drivers::blk;
    use crate::println;

    // reference: https://github.com/rafalh/rust-fatfs/issues/55
    // https://github.com/x37v/stm32h7xx-hal/blob/xnor/fatfs/src/sdmmc.rs#L1392-L1697

    #[derive(Debug)]
    struct AtaError;

    const BSIZE: usize = 512;

    #[derive(Debug)]
    enum DiskCursorIoError {
        UnexpectedEof,
        WriteZero,
    }

    impl IoError for DiskCursorIoError {
        fn is_interrupted(&self) -> bool {
            false
        }

        fn new_unexpected_eof_error() -> Self {
            Self::UnexpectedEof
        }

        fn new_write_zero_error() -> Self {
            Self::WriteZero
        }
    }

    #[derive(Debug, Clone)]
    #[repr(align(4))]
    pub struct DataBlock(pub [u8; BSIZE]);

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

    #[repr(align(4))]
    struct DiskCursor {
        sector: usize,
        offset: usize,
        // cache_sector: Option<usize>,
        block: DataBlock,
    }

    impl DiskCursor {
        fn new(start_sector: usize) -> Self {
            DiskCursor {
                sector: start_sector,
                offset: 0,
                block: DataBlock::new(),
                // cache_sector: None,
            }
        }

        fn get_position(&self) -> usize {
            self.sector * BSIZE + self.offset
        }

        fn set_position(&mut self, position: usize) {
            self.sector = position / BSIZE;
            self.offset = position % BSIZE;
        }

        fn move_cursor(&mut self, amount: usize) {
            self.set_position(self.get_position() + amount)
        }

        fn read_blk(&mut self, start_sector: usize, sector_count: usize) -> Result<(), AtaError> {
            // println!("read_blk: {}, {}", start_sector, sector_count);
            blk::read(start_sector, sector_count, self.block.0.as_ptr() as usize);
            Ok(())
        }

        fn write_blk(&mut self, start_sector: usize, sector_count: usize) -> Result<(), AtaError> {
            // println!("write_blk: {}, {}", start_sector, sector_count);
            blk::write(start_sector, sector_count, self.block.0.as_ptr() as usize);
            Ok(())
        }
    }

    impl IoBase for DiskCursor {
        type Error = DiskCursorIoError;
    }

    impl Read for DiskCursor {
        fn read(&mut self, buf: &mut [u8]) -> Result<usize, DiskCursorIoError> {
            let mut i = 0;
            while i < buf.len() {
                self.read_blk(
                    self.sector,
                    ((buf.len() - i) / BSIZE).max(1).try_into().unwrap(),
                )
                .expect("ata error");
                let data = &self.block.0[self.offset..];
                if data.len() == 0 {
                    break;
                }
                let end = (i + data.len()).min(buf.len());
                let len = end - i;
                buf[i..end].copy_from_slice(&data[..len]);
                i += len;
                self.move_cursor(i);
            }
            Ok(i)
        }

        fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), DiskCursorIoError> {
            let n = self.read(buf)?;
            assert!(n == buf.len(), "TODO: Error");
            Ok(())
        }
    }

    impl Write for DiskCursor {
        fn write(&mut self, buf: &[u8]) -> Result<usize, DiskCursorIoError> {
            let mut i = 0;
            while i < buf.len() {
                // read the block to memory
                self.read_blk(
                    self.sector,
                    ((buf.len() - i) / BSIZE).max(1).try_into().unwrap(),
                )
                .expect("ata error");

                let data = &mut self.block.0[self.offset..];
                if data.len() == 0 {
                    break;
                }
                let end = (i + data.len()).min(buf.len());
                let len = end - i;
                data[..end].copy_from_slice(&buf[i..len]);

                self.write_blk(
                    self.sector,
                    ((buf.len() - i) / BSIZE).max(1).try_into().unwrap(),
                )
                .expect("ata error");

                i += len;
                self.move_cursor(i);
            }
            Ok(i)
        }

        fn flush(&mut self) -> Result<(), DiskCursorIoError> {
            Ok(())
        }
    }

    impl Seek for DiskCursor {
        fn seek(&mut self, pos: SeekFrom) -> Result<u64, DiskCursorIoError> {
            match pos {
                SeekFrom::Start(i) => {
                    self.set_position(i as usize);
                    Ok(i)
                }
                SeekFrom::End(_i) => {
                    unimplemented!()
                }
                SeekFrom::Current(i) => {
                    let new_pos = (self.get_position() as i64) + i;
                    self.set_position(new_pos as usize);
                    Ok(new_pos as u64)
                }
            }
        }
    }

    pub fn test_fatfs() {
        let start_sector: usize = 0;
        let storage = DiskCursor::new(start_sector);

        let options = FsOptions::new();
        let fs = FileSystem::new(storage, options).expect("open fs");
        let root_cursor = fs.root_dir();

        // ls root
        for entry in root_cursor.iter() {
            let entry = entry.expect("Entry");
            print!("{} ", entry.file_name());
        }
        println!();
        println!("-----------");
        // create a file

        let path = "rust_fat32.txt";
        let mut file = root_cursor.create_file(path).expect("file");
        println!("create_file OK");
        file.write(b"fat32 write test").expect("file write");
        drop(file);

        // ls root
        for entry in root_cursor.iter() {
            let entry = entry.expect("Entry");
            print!("{} ", entry.file_name());
        }
        println!();
        println!("-----------");

        let mut file = root_cursor.open_file(path).expect("file");
        let mut buf: [u8; 16] = [0; 16];
        file.read(&mut buf).expect("file read");
        use alloc::string::String;
        let content = String::from_utf8_lossy(&buf);
        println!("CONTENT: {:?}", content);
    }
}
