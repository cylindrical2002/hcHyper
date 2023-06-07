ARCH = riscv64

.PHONY: linux hcminikernel nimbos rCore-Tutorial-v3 clean

hcminikernel:
	@mkdir hcMiniKernel/$(ARCH)-runtime
	@make -C hcMiniKernel/os build LOG=TRACE

nimbos:
	@mkdir nimbos/$(ARCH)-runtime
	@make -C nimbos/user build
	@make -C nimbos/kernel GUEST=on

linux:
	@make -C linux build

rCore-Tutorial-v3:
	@make -C rCore-Tutorial-v3 build

clean:
	@make -C hcMiniKernel/os clean
	@make -C nimbos/kernel clean
	@make -C linux clean
	@make -C rCore-Tutorial-v3 clean