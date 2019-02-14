#![feature(test)]

extern crate test;

use std::mem::size_of;

#[cfg(test)]
mod tests;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ReadError {
    TooManyBits {
        requested: usize,
        max: usize,
    },
    NotEnoughData {
        requested: usize,
        bits_left: usize,
    },
}

pub type Result<T> = std::result::Result<T, ReadError>;

pub struct BitBuffer<'a> {
    bytes: &'a [u8],
    bit_len: usize,
    byte_len: usize,
}

macro_rules! array_ref {
    ($arr:expr, $offset:expr, $len:expr) => {{
        {
            #[inline]
            unsafe fn as_array<T>(slice: &[T]) -> &[T; $len] {
                &*(slice.as_ptr() as *const [_; $len])
            }
            let offset = $offset;
            let slice = & $arr[offset..offset + $len];
            #[allow(unused_unsafe)]
            unsafe {
                as_array(slice)
            }
        }
    }}
}

const USIZE_SIZE: usize = std::mem::size_of::<usize>();

macro_rules! make_signed {
    ($unsigned:expr, $type:ty, $count:expr) => {
        {
            let sign_bits = $unsigned >> ($count - 1) & 1;
            let high_bits = 0 - sign_bits as $type;
            high_bits << $count | $unsigned as $type
        }
    }
}

impl<'a> BitBuffer<'a> {
    pub fn from_padded_slice(bytes: &'a [u8], byte_len: usize) -> BitBuffer<'a> {
        BitBuffer {
            bytes,
            byte_len,
            bit_len: byte_len * 8,
        }
    }

    pub fn bit_len(&self) -> usize {
        self.bit_len
    }

    pub fn byte_len(&self) -> usize {
        self.byte_len
    }

    pub fn read_usize(&self, position: usize, count: usize) -> usize {
        let byte_index = position / 8;
        let bit_offset = position & 7;
        let bytes:&[u8; USIZE_SIZE] = array_ref!(self.bytes, byte_index, USIZE_SIZE);
        let container_le = unsafe {
            std::mem::transmute::<[u8; USIZE_SIZE], usize>(*bytes)
        };
        let container = usize::from_le(container_le);
        let shifted = container >> bit_offset;
        let mask = !(usize::max_value() << count);
        shifted & mask
    }

    pub fn read_u8(&self, position: usize, count: usize) -> u8 {
        self.read_usize(position, count) as u8
    }

    pub fn read_u16(&self, position: usize, count: usize) -> u16 {
        self.read_usize(position, count) as u16
    }

    pub fn read_u32(&self, position: usize, count: usize) -> u32 {
        if size_of::<usize>() > size_of::<u32>() {
            self.read_usize(position, count) as u32
        } else {
            let value:u32 = (self.read_u16(position, count) as u32) << 16;
            value | self.read_u16(position + 16, count - 16) as u32
        }
    }
    pub fn read_i8(&self, position: usize, count: usize) -> i8 {
        let unsigned = self.read_u8(position, count);
        make_signed!(unsigned, i8, count)
    }

    pub fn read_i16(&self, position: usize, count: usize) -> i16 {
        let unsigned = self.read_u16(position, count);
        make_signed!(unsigned, i16, count)
    }

    pub fn read_i32(&self, position: usize, count: usize) -> i32 {
        let unsigned = self.read_u32(position, count);
        make_signed!(unsigned, i32, count)
    }
}