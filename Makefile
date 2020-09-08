image_file = woodyOS.img
ipl_file = ipl10
os_file = haribote
asm_head = asmhead

${ipl_file}.bin: ${ipl_file}.nas
	nasm ${ipl_file}.nas -o ${ipl_file}.bin -l ${ipl_file}.lst
${asm_head}.bin: ${asm_head}.nas
	nasm ${asm_head}.nas -o ${asm_head}.bin -l ${asm_head}.lst
kernel.bin: src/lib.rs
	cargo xbuild --target i686-haribote.json
	ld -v -nostdlib -m elf_i386 -Tdata=0x00310000 -Tkernel.ld ./target/i686-haribote/debug/libharibote_os.a -o kernel.bin
${os_file}.sys: kernel.bin ${asm_head}.bin
	cat ${asm_head}.bin kernel.bin > ${os_file}.sys
${image_file}: ${os_file}.sys ${ipl_file}.bin
	mformat -f 1440 -B ${ipl_file}.bin -C -i woodyOS.img ::
	mcopy ${os_file}.sys -i woodyOS.img ::

img: ${image_file}
run: img
	qemu-system-x86_64 -drive format=raw,if=floppy,file=${image_file}
clean:
	rm -f *.bin *.lst *.img *.sys
