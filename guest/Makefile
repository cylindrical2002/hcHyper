ARCH = riscv64

build_dir := build/$(ARCH)/

build_dir:
	@mkdir -p $(build_dir)

hcminikernel: build_dir
	@mkdir $(build_dir)/hcminikernel/
	@make -C hcMiniKernel/os build LOG=TRACE
	@echo "finished building hcMiniKernel"

nimbos: build_dir
	@mkdir $(build_dir)/nimbos/
	@make -C nimbos/user build
	@make -C nimbos/kernel GUEST=on

linux: build_dir
	dtc -O dtb -o linux/linux.dtb linux/linux.dts

rCore-Tutorial-v3: build_dir
	dtc -O dtb -o rCore-Tutorial-v3/rCore-Tutorial-v3.dtb rCore-Tutorial-v3/rCore-Tutorial-v3.dts

clean:
	@make -C hcMiniKernel/os clean
	@make -C nimbos/kernel clean
	@rm linux/linux.dtb
	@rm rCore-Tutorial-v3/rCore-Tutorial-v3.dtb
	@rm -rf build/