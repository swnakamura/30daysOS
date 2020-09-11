image_file = woodyOS.img
ipl_file = ipl10
os_file = haribote
asm_head = asmhead
kernel = kernel

asm_dir = asm
output_dir = build

${output_dir}/${ipl_file}.bin: ${asm_dir}/${ipl_file}.nas
	nasm $^ -o $@ -l ${output_dir}/${ipl_file}.lst
${output_dir}/${asm_head}.bin: ${asm_dir}/${asm_head}.nas
	nasm $^ -o $@ -l ${output_dir}/${asm_head}.lst

${output_dir}/${kernel}.bin: src/lib.rs
	cargo xbuild --target i686-haribote.json
	ld -v -nostdlib -m elf_i386 -Tdata=0x00310000 -Tkernel.ld ./target/i686-haribote/debug/libharibote_os.a -o $@

${output_dir}/${os_file}.sys: ${output_dir}/${asm_head}.bin ${output_dir}/${kernel}.bin
	cat $^ > $@

${output_dir}/${image_file}: ${output_dir}/${os_file}.sys ${output_dir}/${ipl_file}.bin
	mformat -f 1440 -B ${output_dir}/${ipl_file}.bin -C -i $@ ::
	mcopy $< -i $@ ::

img:   ${output_dir}/${image_file}
build: ${output_dir}/${image_file}

run: img
	qemu-system-x86_64 -drive format=raw,if=floppy,file=${output_dir}/${image_file}
clean:
	rm -f build/*

.PHONY: img run clean
