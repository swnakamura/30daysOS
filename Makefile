image_file = woodyOS.img
ipl_file = ipl10
os_file = haribote

${os_file}.sys: ${os_file}.nas
	nasm ${os_file}.nas -o ${os_file}.sys
${ipl_file}.bin: ${ipl_file}.nas
	nasm ${ipl_file}.nas -o ${ipl_file}.bin -l ${ipl_file}.lst
${image_file}: ${os_file}.sys ${ipl_file}.bin
	mformat -f 1440 -B ${ipl_file}.bin -C -i woodyOS.img ::
	mcopy ${os_file}.sys -i woodyOS.img ::

img: ${image_file}
run: img
	qemu-system-x86_64 -drive format=raw,if=floppy,file=${image_file}
clean:
	rm *.bin *.lst *.img *.sys
