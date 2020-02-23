./qemu-system-riscv64 \
    -nographic -machine virt \
    -m 128 \
    -bios "$@"
