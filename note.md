# 情報の単位

| 単位        | /BYTE      | /bit  | /pattern   | example    |
| ----------- | ---------: | ---:  | ---------: | ---------- |
| 16進一桁    | 0.5        | 4     | 16         |            |
| BYTE        | 1          | 8     | 256        | char=ASCII |
| WORD        | 2          | 16    | 65536      | short      |
| DOUBLE WORD | 4          | 32    | 4294967296 | int        |
| 4KiB        | 4096=16**3 | 32768 | 膨大       | 1ページ    |

# CPUのレジスタ

## 16bitレジスタ

* AX: accumulator. 演算に便利
* CX: counter. カウントに便利
* DX: data
* BX: base
* SP: stack pointer
* BP: base pointer
* SI: source（読み込み）index
* DI: destination（書き込み）index

Xはextendedの略

### FLAGSレジスタ
特別な16bitレジスタ。フラグ情報が詰まっている。
```
| 15 | 14 | 13   | 12 | 11 | 10 | 9  | 8  | 7  | 6  | 5 | 4  | 3 | 2  | 1 | 0  |
|    | NT | IOPL      | OF | DF | IF | TF | SF | ZF |   | AF |   | PF |   | CF |
```
* CF: キャリーフラグ
* IF: 割り込みフラグ

### セグメントレジスタ

* ES: extra segment
* CS: code segment
* SS: stack segment
* DS: data segment
* FS: 本名なし
* GS: 本名なし

セグメンテーションに使う。BIOSなどの16bitモードでは16倍して番地に足す(p.52)ことでメモリを1MBまで水増しするが、32bitモードではその表すセグメントの開始番地を足すことでずらす(p.112)。

### GDTR

Global (segment) Descriptor Table（大域セグメント記述子表）を定義するために使う。
セグメントのためには、

* セグメントの大きさ
* 開始番地
* 属性

をまとめた8B=64bitの情報(Segment Descriptor)を管理する必要がある。しかし、32bitモードでもセグメントレジスタは16bitのままであるので足りない。
なのでテーブルを使う。セグメント番号のみをセグメントレジスタは記憶し、具体的にどのようなSegment Descriptorなのかはその番号から参照して用いる。

セグメントレジスタは16bitなので、CPUの仕様上使えない下3bitを除いた13bit=[0,8191]がセグメント番号として使える。
それぞれの番号に対応するSegment Descriptorの実際の内容はメモリに書き込まれる。それぞれ8Bより8192*8B=64KB必要となる。これをGDTと言う。
GDTはどこにおいてもいいが、単純に連続したメモリにするとして、その先頭番地と設定の個数くらいはCPU側でも覚えておく必要があるだろう。

これを覚えておくのがGDTR (- Register)。合計48bitの特別なレジスタ。

```
|47|46|45|44|43|42|41|40|39|38|37|36|35|34|33|32|31|30|29|28|27|26|25|24|23|22|21|20|19|18|17|16|15|14|13|12|11|10|9|8|7|6|5|4|3|2|1|0|
|Base                                                                                           |Limit                                |
```

* Base: GDTのある番地
* Limit: GDTの有効バイト数-1。2^16B = 64KBなので、確かにこれだけあれば最大値まで記述可能。

なお、セグメントの内容は以下の通り([Wikipedia-Segment_descriptor])。たしかに64bit。
```
|31|30|29|28|27|26|25|24|23|22|21|20 |19|18|17|16         |15|14|13|12|11  |10 |9  |8|7|6|5|4|3|2|1|0    |
|Base Address[31:24]    |G |D |L |AVL|Segment Limit[19:16]|P |DPL  |1 |Type|C/E|R/W|A|Base Address[23:16]|
|Base Address[15:0]                                       |Segment Limit[15:0]                           |
```

[Wikipedia]: https://en.wikipedia.org/wiki/Segment_descriptor

### IDTR

GDTRと同様に、割り込みを記述するテーブルIDTと、その情報を表すレジスタ。ただし、IDTRは8bit=[0,255]より256個まで。よって256*8B=2KB。

IDTの内容は以下の通り ( [OSDev](https://wiki.osdev.org/Interrupt_Descriptor_Table) ) 。
```
|31|30|29|28|27|26|25|24|23|22|21|20|19|18|17|16|15|14|13|12|11|10|9|8|7|6|5|4|3|2|1|0|
|Offset[31:16]                                  |P |DPL  |S |Type     |0|0|0|0|0|0|0|0|
|Selector[15:0]                                 |Offset[15:0]                         |
```

## 8bitレジスタ

16bitレジスタの一部。

* AL: accumulator low
* CL: counter low
* DL: data low
* BL: base low
* AH: accumulator high
* CH: counter high
* DH: data high
* BH: base high

SP,BP,SI,DIについてはこれらのL/Hレジスタはないので、もし使いたければこれらのいずれかに一度コピーしてアクセスする

## 32bitレジスタ

* EAX
* ECX
* EDX
* EBX
* ESP
* EBP
* ESI
* EDI

Eはextendedの略。

また、8bitのときと同様、16bitレジスタは32bitレジスタの一部。例えばAXはEAXの下位16bit。32bitレジスタの上位16bitにアクセスする方法はない。もし使いたい場合、シフトして下位16bitを取れば良い

