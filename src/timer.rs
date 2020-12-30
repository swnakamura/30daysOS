use spin::Mutex;
use x86_64::instructions::port;

const PIT_CTRL: u16 = 0x0043;
const PIT_CNT0: u16 = 0x0040;

pub struct TIMERCTL {
    pub count: i32,
}

pub static TIMER_CONTROL: Mutex<TIMERCTL> = Mutex::new(TIMERCTL { count: 0 });

pub fn init_pit() {
    let mut port_control = port::PortWriteOnly::new(PIT_CTRL);
    let mut port_counter = port::PortWriteOnly::new(PIT_CNT0);
    unsafe {
        port_control.write(0x34u8);
        port_counter.write(0x9cu8);
        port_counter.write(0x2eu8);
    }
}
