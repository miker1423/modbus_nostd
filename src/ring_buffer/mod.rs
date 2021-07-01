use core::cmp;
use managed::ManagedSlice;

pub enum Error {
    Exhausted,
}

#[derive(Debug)]
pub struct RingBuffer<'a, T: 'a> {
    storage: ManagedSlice<'a, T>,
    read_at: usize,
    length: usize,
}


impl<'a> RingBuffer<'a, u8> {
    pub fn new(buffer:&'a mut [u8; 256]) -> RingBuffer<'a, u8>
    {
        RingBuffer {
            storage: ManagedSlice::Borrowed(buffer),
            read_at: 0,
            length: 0
        }
    }

    pub fn clear(&mut self){
        self.read_at = 0;
        self.length = 0;
    }

    pub fn capacity(&self) -> usize {
        self.storage.len()
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn window(&self) -> usize {
        self.capacity() - self.len()
    }

    pub fn contiguous_window(&self) -> usize {
        cmp::min(self.window(), self.capacity() - self.get_idx(self.length))
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn is_full(&self) -> bool {
        self.window() == 0
    }

    fn get_idx(&self, idx: usize) -> usize {
        let len = self.capacity();
        if len > 0 {
            (self.read_at + idx) % len
        } else {
            0
        }
    }

    fn get_idx_unchecked(&self, idx: usize) -> usize {
        (self.read_at + idx) % self.capacity()
    }

    pub fn push_single(&mut self, value: u8) -> Result<(), Error> {
        if self.is_full() { return Err(Error::Exhausted) }
        let index = self.get_idx_unchecked(self.length);
        self.storage[index] = value;
        self.length += 1;
        Ok(())
    }

    pub fn push<'b, R, F>(&'b mut self, f: F) -> Result<R, Error>
        where F: FnOnce(&'b mut u8) -> Result<R, Error> {
        if self.is_full() { return Err(Error::Exhausted) }

        let index = self.get_idx_unchecked(self.length);
        match f(&mut self.storage[index]) {
            Ok(result) => {
                self.length += 1;
                Ok(result)
            }
            Err(error) => Err(error)
        }
    }

    pub fn get_written(&self) -> &[u8] {
        &self.storage[..self.len()]
    }
}