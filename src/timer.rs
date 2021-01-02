use crate::FIFO;
use spin::Mutex;
use x86_64::instructions::port;

const PIT_CTRL: u16 = 0x0043;
const PIT_CNT0: u16 = 0x0040;

const TIMER_FIFO_SIZE: usize = 32;

#[derive(Debug)]
pub struct TIMERCTL<T> {
    pub count: i32,
    pub timeout: i32,
    pub fifo: FIFO<T>,
    data: T,
}

impl<T: Copy> TIMERCTL<T> {
    /// pushes `self.data` to `self.fifo` in order to notify the timeout.
    pub fn push_timeout_signal(&mut self) {
        self.fifo.push(self.data).unwrap();
    }
    pub fn set_timer(&mut self, timeout: i32) {
        self.timeout = timeout;
    }
}

use lazy_static::lazy_static;
lazy_static! {
    pub static ref TIMER_CONTROL: Mutex<TIMERCTL<u8>> = Mutex::new(TIMERCTL {
        count: 0,
        timeout: 0,
        fifo: FIFO::new(TIMER_FIFO_SIZE, 0),
        data: 0,
    });
}

pub fn init_pit() {
    let mut port_control = port::PortWriteOnly::new(PIT_CTRL);
    let mut port_counter = port::PortWriteOnly::new(PIT_CNT0);
    unsafe {
        port_control.write(0x34u8);
        port_counter.write(0x9cu8);
        port_counter.write(0x2eu8);
    }
}
