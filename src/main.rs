mod nor_flash_storage;

use nor_flash_storage::nor_flash::{ErrorType, NorFlash, NorFlashError, NorFlashErrorKind, ReadNorFlash};

/// In-memory NOR flash simulator: erase sets bytes to 0xFF, write can only clear bits (AND).
struct MockNorFlash<const CAPACITY: usize, const ERASE: usize> {
    data: [u8; CAPACITY],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MockError {
    OutOfBounds,
    NotAligned,
}

impl<const CAPACITY: usize, const ERASE_SIZE: usize> MockNorFlash<CAPACITY, ERASE_SIZE> {
    pub fn dump_data(&self) {
        for chunk in self.data.chunks(ERASE_SIZE) {
            for byte in chunk {
                print!(" {:02X}", byte);
            }
            println!();
        }
    }
}

impl NorFlashError for MockError {
    fn kind(&self) -> NorFlashErrorKind {
        match self {
            MockError::OutOfBounds => NorFlashErrorKind::OutOfBounds,
            MockError::NotAligned => NorFlashErrorKind::NotAligned,
        }
    }
}

impl<const CAPACITY: usize, const ERASE_SIZE: usize> MockNorFlash<CAPACITY, ERASE_SIZE> {
    fn new() -> Self {
        assert!(ERASE_SIZE > 0 && CAPACITY % ERASE_SIZE == 0);
        Self { data: [0xff; CAPACITY] }
    }
}

impl<const CAPACITY: usize, const ERASE_SIZE: usize> ErrorType for MockNorFlash<CAPACITY, ERASE_SIZE> {
    type Error = MockError;
}

impl<const CAPACITY: usize, const ERASE_SIZE: usize> ReadNorFlash for MockNorFlash<CAPACITY, ERASE_SIZE> {
    const READ_SIZE: usize = 1;

    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        let offset_start = offset as usize;
        let offset_end = offset_start.checked_add(bytes.len()).ok_or(MockError::OutOfBounds)?;
        if offset_end > CAPACITY {
            return Err(MockError::OutOfBounds);
        }
        bytes.copy_from_slice(&self.data[offset_start..offset_end]);
        Ok(())
    }

    fn capacity(&self) -> usize {
        CAPACITY
    }
}

impl<const CAPACITY: usize, const ERASE_SIZE: usize> NorFlash for MockNorFlash<CAPACITY, ERASE_SIZE> {
    const WRITE_SIZE: usize = 1;
    const ERASE_SIZE: usize = ERASE_SIZE;

    fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        let (offset_from, offset_to) = (from as usize, to as usize);
        if offset_from % ERASE_SIZE != 0 || offset_to % ERASE_SIZE != 0 {
            return Err(MockError::NotAligned);
        }
        if offset_from > offset_to || offset_to > CAPACITY {
            return Err(MockError::OutOfBounds);
        }
        self.data[offset_from..offset_to].fill(0xff);
        Ok(())
    }

    fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        let start = offset as usize;
        let end = start.checked_add(bytes.len()).ok_or(MockError::OutOfBounds)?;
        if end > CAPACITY {
            return Err(MockError::OutOfBounds);
        }
        for (dst, src) in self.data[start..end].iter_mut().zip(bytes) {
            *dst &= *src;
        }
        Ok(())
    }
}

fn main() {
    const CAPACITY: usize = 256;
    const ERASE_SIZE: usize = 32;
    let mut buf_cache = [0u8; ERASE_SIZE];
    let mut storage = nor_flash_storage::NorFlashStorage::new(MockNorFlash::<CAPACITY, ERASE_SIZE>::new(), &mut buf_cache);
    let mut buf_write_16bytes = [0u8; 16];
    for (i, bufp) in buf_write_16bytes.iter_mut().enumerate() { *bufp = i as u8; }
    let mut buf_write_64bytes = [0u8; 64];
    for (i, bufp) in buf_write_64bytes.iter_mut().enumerate() { *bufp = i as u8; }
    {
        let mut buf_read = [0u8; 16];
        storage.write(0, &buf_write_16bytes).unwrap();
        storage.read(0, &mut buf_read).unwrap();
        println!("{:02X?}", buf_read);
        storage.nor_flash().dump_data();
    }
    {
        let mut buf_read = [0u8; 16];
        storage.write(8, &buf_write_16bytes).unwrap();
        storage.read(8, &mut buf_read).unwrap();
        println!("{:02X?}", buf_read);
        storage.nor_flash().dump_data();
    }
    {
        let mut buf_read = [0u8; 16];
        storage.write(32, &buf_write_16bytes).unwrap();
        storage.read(32, &mut buf_read).unwrap();
        println!("{:02X?}", buf_read);
        storage.nor_flash().dump_data();
    }
    {
        let mut buf_read = [0u8; 16];
        storage.write(64, &buf_write_16bytes).unwrap();
        storage.read(64, &mut buf_read).unwrap();
        println!("{:02X?}", buf_read);
        storage.nor_flash().dump_data();
    }
    {
        let mut buf_read = [0u8; 64];
        storage.write(96, &buf_write_64bytes).unwrap();
        storage.read(96, &mut buf_read).unwrap();
        println!("{:02X?}", buf_read);
        storage.nor_flash().dump_data();
    }
    {
        println!();
        storage.sync().unwrap();
        storage.nor_flash().dump_data();
    }
    {
        let mut buf_read = [0u8; 64];
        storage.write(176, &buf_write_64bytes).unwrap();
        storage.read(176, &mut buf_read).unwrap();
        println!("{:02X?}", buf_read);
        storage.nor_flash().dump_data();
    }
}
