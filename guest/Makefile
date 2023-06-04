ARCH = riscv64

hcminikernel:
	@mkdir hcMiniKernel/$(ARCH)-runtime
	@make -C hcMiniKernel/os build LOG=TRACE

nimbos: build_dir
	@mkdir nimbos/$(ARCH)-runtime
	@make -C nimbos/user build
	@make -C nimbos/kernel GUEST=on

linux: build_dir
	@make -C linux build

rCore-Tutorial-v3: build_dir
	dtc -O dtb -o rCore-Tutorial-v3/rCore-Tutorial-v3.dtb rCore-Tutorial-v3/rCore-Tutorial-v3.dts

clean:
	@make -C hcMiniKernel/os clean
	@make -C nimbos/kernel clean
	@make -C linux clean
	@make -C rCore-Tutorial-v3 clean