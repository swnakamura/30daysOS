image_file = woodyOS.img
ipl_file = ipl10

haribote.sys: haribote.nas
	nasm haribote.nas -o haribote.sys
${ipl_file}.bin: ${ipl_file}.nas
	nasm ${ipl_file}.nas -o ${ipl_file}.bin -l ${ipl_file}.lst
${image_file}: haribote.sys ${ipl_file}.bin
	mformat -f 1440 -B ${ipl_file}.bin -C -i woodyOS.img ::
	mcopy haribote.sys -i woodyOS.img ::

img: ${image_file}
run: img
	qemu-system-x86_64 -drive format=raw,if=floppy,file=${image_file}
clean:
	rm *.bin *.lst *.img *.sys
