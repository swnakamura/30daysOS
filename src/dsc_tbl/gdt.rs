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
