# 情報の単位

| 単位        | /BYTE | /bit | /pattern   | example    |
| ----------- | ----: | ---: | ---------: | ---------- |
| 16進一桁    | 0.5   | 4    | 16         |            |
| BYTE        | 1     | 8    | 256        | char=ASCII |
| WORD        | 2     | 16   | 65536      | short      |
| DOUBLE WORD | 4     | 32   | 4294967296 | int        |

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

Xはextendの略

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

セグメンテーションに使う。BIOSなどの16itモードでは16倍して番地に足す(p.52)ことでメモリを1MBまで水増しするが、32bitモードではその表すセグメントの開始番地を足すことでずらす(p.112)。

### GDTR

global (segment) descriptor table（割り込み記述子表）を定義するために使う。
セグメントのためには、

* セグメントの大きさ
* 開始番地
* 属性

をまとめた8B=64bitの情報を管理する必要がある。しかし、32bitモードでもセグメントレジスタは16bitのままであるので足りない。

なのでテーブルを使う。セグメント番号のみをセグメントレジスタは記憶し、その番号がどのセグメントに対応するのかを予め設定しておく。
セグメントレジスタは16bitなので、CPUの使用上使えない下3bitを除いた13bit=[0,8191]がセグメント番号として使える。

セグメントの実際の内容はメモリに書き込まれる。それぞれ8Bより8192*8B=64KBとなるが、これをGDTと言う。
GDTはどこにおいてもいいが、その先頭番地と現在有効な設定個数は覚えておく必要があるだろう。これを覚えておくのがGDTR (- register)。

### IDTR

GDTRと同様に、割り込みを記述するテーブル。[0,255]。よって256*8B=2KB。

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

Eはextendの略。

また、8bitのときと同様、16bitレジスタは32bitレジスタの一部。例えばAXはEAXの下位16bit。32bitレジスタの上位16bitにアクセスする方法はない。もし使いたい場合、シフトして下位16bitを取れば良い

