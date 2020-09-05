os: ipl.nas
	nasm ipl.nas -o ipl.bin -l ipl.lst
run: os
	qemu-system-x86_64 -drive format=raw,if=floppy,file=ipl.bin
clean:
	rm *.bin *.lst
