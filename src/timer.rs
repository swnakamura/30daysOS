use crate::FIFO;
use alloc::{vec, vec::Vec};
use spin::Mutex;
use x86_64::instructions::port;

const TIMER_FIFO_SIZE: usize = 32;
const MAX_TIMER: usize = 100;

pub struct TIMERCTL<T: Copy> {
    pub count: i32,
    pub next: i32,
    pub timers: Vec<TIMER<T>>,
}

impl<T: Copy> TIMERCTL<T> {
    pub fn allocate(&mut self) -> Option<usize> {
        for i in 0..MAX_TIMER {
            if self.timers[i].flag == TimerState::Unused {
                self.timers[i].flag = TimerState::Using;
                return Some(i);
            }
        }
        return None;
    }
    pub fn deallocate(&mut self, id: usize) {
        self.timers[id].flag = TimerState::Unused;
    }
    pub fn set_time(&mut self, id: usize, wait_time: i32) {
        let timeout = self.count + wait_time;
        self.timers[id].timeout = timeout;
        self.timers[id].flag = TimerState::Using;
        self.next = core::cmp::min(self.next, timeout);
    }
    pub fn refresh_nexts(&mut self) {
        self.next = core::i32::MAX;
        for timer in &mut self.timers {
            if timer.flag == TimerState::Using {
                if self.count == timer.timeout {
                    timer.push_timeout_signal();
                    timer.flag = TimerState::Allocated;
                } else {
                    self.next = core::cmp::min(self.next, timer.timeout);
                }
            }
        }
    }
    pub fn check_timers(&mut self) {
        if self.next == self.count {
            self.refresh_nexts();
        }
    }
}

use lazy_static::lazy_static;
lazy_static! {
    pub static ref TIMER_CONTROL: Mutex<TIMERCTL<u8>> = Mutex::new(TIMERCTL {
        count: 0,
        next: core::i32::MAX,
        timers: vec![TIMER::new(0); MAX_TIMER],
    });
}

#[derive(Clone)]
pub struct TIMER<T: Copy> {
    pub timeout: i32,
    pub flag: TimerState,
    fifo: FIFO<T>,
    pub data: T,
}

#[derive(Clone, PartialEq, Eq)]
pub enum TimerState {
    Unused,
    Allocated,
    Using,
}

impl<T: Copy> TIMER<T> {
    pub fn new(data: T) -> Self {
        Self {
            timeout: 0,
            flag: TimerState::Unused,
            fifo: FIFO::new(TIMER_FIFO_SIZE, data),
            data,
        }
    }
    /// pushes `self.data` to `self.fifo` in order to notify the timeout.
    pub fn push_timeout_signal(&mut self) {
        self.fifo.push(self.data).unwrap();
    }
    pub fn pop(&mut self) -> Result<T, ()> {
        self.fifo.pop()
    }
    pub fn deallocate(&mut self) {
        self.flag = TimerState::Unused;
    }
}

const PIT_CTRL: u16 = 0x0043;
const PIT_CNT0: u16 = 0x0040;

pub fn init_pit() {
    let mut port_control = port::PortWriteOnly::new(PIT_CTRL);
    let mut port_counter = port::PortWriteOnly::new(PIT_CNT0);
    unsafe {
        port_control.write(0x34u8);
        port_counter.write(0x9cu8);
        port_counter.write(0x2eu8);
    }
}
