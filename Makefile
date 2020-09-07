image_file = woodyOS.img

haribote.sys: haribote.nas
	nasm haribote.nas -o haribote.sys
ipl.bin: ipl.nas
	nasm ipl.nas -o ipl.bin -l ipl.lst
${image_file}: haribote.sys ipl.bin
	mformat -f 1440 -B ipl.bin -C -i woodyOS.img ::
	mcopy haribote.sys -i woodyOS.img ::

img: ${image_file}
run: img
	qemu-system-x86_64 -drive format=raw,if=floppy,file=${image_file}
clean:
	rm *.bin *.lst *.img *.sys
