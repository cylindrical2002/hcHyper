ARCH = riscv64

build:
	@dtc -O dtb -o $(ARCH)-runtime/linux.dtb linux-$(ARCH).dts

clean:
	@rm -f riscv64-runtime/linux.dtb
	@rm -f aarch64-runtime/linux.dtb