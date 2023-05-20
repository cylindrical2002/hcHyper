ARCH = riscv64

build_dir := build/$(ARCH)/

guests := 

ifeq ($(ARCH), riscv64) 
	guests += hcminikernel
endif

build: $(guests)
	@echo "build $(guests) successful"

build_dir:
	@mkdir -p $(build_dir)

hcminikernel: build_dir
	@make -C hcMiniKernel/os build LOG=TRACE
	@echo "finished building hcMiniKernel"

clean:
	@make -C hcMiniKernel/os clean
	@rm -rf $(build_dir)