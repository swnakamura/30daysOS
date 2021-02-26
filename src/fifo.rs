use alloc::{vec, vec::Vec};

/// uses static-sized vector as a buffer
#[derive(Debug, Clone)]
pub struct FIFO<T> {
    buf: Vec<T>,
    p: usize,
    q: usize,
    size: usize,
    free: usize,
}

pub const BUF_SIZE: usize = 2048;

pub const KEYBOARD_OFFSET: u32 = 256;
pub const MOUSE_OFFSET: u32 = 512;

use lazy_static::lazy_static;
use spin::Mutex;
lazy_static! {
    /// Unified FIFO buffer of haribote OS.
    /// 0: ticking x
    /// 1: ticking y
    /// 3: 3 sec have passed
    /// 10: 10 sec have passed
    /// 256-511: keyboard input (offset 256)
    /// 512-767: mouse input (offset 512)
    pub static ref GLOBAL_FIFO_BUF: Mutex<FIFO<u32>> = Mutex::new(FIFO::new(BUF_SIZE, 0));
}

impl<T: Clone> FIFO<T> {
    pub fn new(buf_size: usize, default_value: T) -> Self {
        Self {
            buf: vec![default_value; buf_size],
            p: 0,
            q: 0,
            free: buf_size,
            size: buf_size,
        }
    }
    pub fn push(&mut self, data: T) -> Result<(), ()> {
        if self.free == 0 {
            return Err(());
        }
        self.buf[self.p] = data;
        self.p += 1;
        if self.p == self.size {
            self.p = 0;
        }
        self.free -= 1;
        Ok(())
    }
    pub fn pop(&mut self) -> Result<T, ()> {
        if self.free == self.size {
            return Err(());
        }
        let data = self.buf[self.q].clone();
        self.q += 1;
        if self.q == self.size {
            self.q = 0;
        }
        self.free += 1;
        return Ok(data);
    }
    pub fn status(&self) -> usize {
        return self.size - self.free;
    }
}
