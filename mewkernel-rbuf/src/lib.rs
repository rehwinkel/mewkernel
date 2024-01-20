#![no_std]

pub struct RingBuffer<const SIZE: usize> {
    read: usize,
    write: usize,
    buffer: [u8; SIZE],
}

impl<const SIZE: usize> RingBuffer<SIZE> {
    pub const fn new() -> Self {
        Self {
            buffer: [0; SIZE],
            read: 0,
            write: 0,
        }
    }

    pub fn read(&mut self) -> Option<u8> {
        if self.read == self.write {
            None
        } else {
            let value = self.buffer[self.read];
            self.read = (self.read + 1) % SIZE;
            Some(value)
        }
    }

    pub fn write(&mut self, value: u8) -> Option<()> {
        let next_write = (self.write + 1) % SIZE;
        if next_write == self.read {
            None
        } else {
            self.buffer[self.write] = value;
            self.write = next_write;
            Some(())
        }
    }

    pub fn is_empty(&self) -> bool {
        self.read == self.write
    }

    pub fn is_full(&self) -> bool {
        (self.write + 1) % SIZE == self.read
    }

    pub fn len(&self) -> usize {
        if self.write >= self.read {
            self.write - self.read
        } else {
            SIZE - self.read + self.write
        }
    }

    pub fn write_slice(&mut self, slice: &[u8]) -> Result<(), usize> {
        if self.len() + slice.len() >= SIZE {
            Err(self.len() + slice.len() - SIZE + 1)
        } else {
            for value in slice {
                self.write(*value).unwrap();
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests;
