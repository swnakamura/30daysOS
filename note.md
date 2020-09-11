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

## セグメントレジスタ

* ES: extra segment
* CS: code segment
* SS: stack segment
* DS: data segment
* FS: 本名なし
* GS: 本名なし
