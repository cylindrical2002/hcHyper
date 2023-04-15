build_dir := build/riscv64/

build: hcminikernel

build_dir:
	@mkdir -p $(build_dir)

hcminikernel: build_dir
	@make -C hcMiniKernel/os build
	@echo "finished building hcMiniKernel"

clean:
	@make -C hcMiniKernel/os clean
	@rm -rf $(build_dir)