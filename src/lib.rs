#![no_std]
#![feature(asm)]
#![feature(start)]

use core::panic::PanicInfo;

use core::fmt::Write;

fn hlt() {
    unsafe {
        asm!("HLT");
    }
}
mod font;
mod io_func;
mod vga;
mod descriptor_table {
    mod gdt {
        /// 全部で8Byteの構造体。目的などについてnote.mdも参照すること。
        ///
        /// base=セグメントの番地は、互換性のためlow,mid,highの3箇所に切り分けられて格納される。合計2+1+1=4Byte
        ///
        /// limit: セグメントが何バイトであるかを表す。しかし、20bitしかないので、セグメントあたり2^20B=1MBまでしかメモリを指定できないような気がする。
        /// ところが、セグメントの属性にGビットというのがあり、これをセットするとlimitはバイト単位ではなくページ＝4KB単位であると解釈される。よってその4000倍の4GBまで指定できる。
        /// これは、limit_lowとlimit_highに書き込まれるが、limit_highの上位4bitは実際にはセグメント属性用のため、たしかに16+8-4=20bit。
        ///
        /// access_rightまたはar: セグメントの属性/アクセス権。
        /// GD00xxxxxxxxという構成。GD00の部分は実際は先述のようにlimit_highの上位4bitという離れた場所に格納されていることに注意。この4bitは386以降に出てきたので拡張アクセス権と呼ばれる。
        /// G: Gビット
        /// D: セグメントのモード。1だと32bit、0だと16bitモードになる。16bitモードといっても80286のプログラムを動かす用であり、BIOSを呼び出すのに使えるわけではない。なので普段は1。
        #[repr(C, packed)]
        pub struct SegmentDescriptor {
            limit_low: u16,
            base_low: u16,
            base_mid: u8,
            access_right: u8,
            limit_high: u8,
            base_high: u8,
        }
        impl SegmentDescriptor {
            /// 定義に従って代入する
            pub fn new(mut limit: u32, mut base: u32, mut ar: u16) -> Self {
                if limit > 0xfffff {
                    // limitが1MBより大きいときはGビットを使う
                    ar |= 0x8000;
                    limit /= 0x1000;
                }
                SegmentDescriptor {
                    limit_low: (limit & 0xffff) as u16,
                    base_low: (base & 0xffff) as u16,
                    base_mid: ((base >> 16) & 0xff) as u8,
                    access_right: (ar & 0xff) as u8,
                    limit_high: (((ar >> 8) & 0xf0) as u8) | (((limit >> 16) & 0x0f) as u8),
                    base_high: ((base >> 24) & 0xff) as u8,
                }
            }
        }

        struct Dtr {
            limit: u16,
            addr: u32,
        }
        /// GDTRを設定する
        pub fn load_gdtr(limit: u16, addr: u32) {
            let gdtr = Dtr { limit, addr };
            unsafe {
                #[cfg(feature = "inline_asm")]
                llvm_asm!("LGDT ($0)" :: "r" (gdtr) : "memory");
            }
        }
    }
    mod idt {
        #[repr(C, packed)]
        pub struct GateDescriptor {
            offset_low: u16,
            selector: u16,
            dw_count: u8,
            access_right: u8,
            offset_high: u16,
        }
        impl GateDescriptor {
            pub fn new(offset: u32, selector: u16, ar: u32) -> Self {
                GateDescriptor {
                    offset_low: (offset & 0xffff) as u16,
                    selector,
                    dw_count: (ar >> 8) as u8,
                    access_right: (ar & 0xff) as u8,
                    offset_high: (offset >> 16) as u16,
                }
            }
        }
        struct Dtr {
            limit: u16,
            addr: u32,
        }
        pub fn load_idtr(limit: u16, addr: u32) {
            let idtr = Dtr { limit, addr };
            unsafe {
                #[cfg(feature = "inline_asm")]
                llvm_asm!("LIDT ($0)" :: "r" (gdtr) : "memory");
            }
        }
    }

    use gdt::*;
    use idt::*;

    const GDT_ADDR: u32 = 0x00270000;
    const IDT_ADDR: u32 = 0x0026f800;

    pub fn init_gdtidt() {
        let gdt = GDT_ADDR as *mut SegmentDescriptor;

        unsafe {
            for i in 0..8192 {
                *gdt.offset(i) = SegmentDescriptor::new(0, 0, 0);
            }
            // メモリ全体
            *gdt.offset(1) = SegmentDescriptor::new(0xffffffff, 0x00000000, 0x4092);
            // bootpackのため
            *gdt.offset(2) = SegmentDescriptor::new(0x0007ffff, 0x00280000, 0x409a);
        }
        load_gdtr(0xffff, GDT_ADDR);

        let idt = IDT_ADDR as *mut GateDescriptor;
        unsafe {
            for i in 0..256 {
                *idt.offset(i) = GateDescriptor::new(0, 0, 0);
            }
        }
        load_idtr(0x7ff, 0x0026f800);

        return;
    }
}

#[no_mangle]
#[start]
pub extern "C" fn haribote_os() -> ! {
    let mut screen = vga::Screen::new();
    screen.init();

    descriptor_table::init_gdtidt();

    loop {
        hlt()
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        hlt()
    }
}
