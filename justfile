default: kernel run

kernel_path := "./target/riscv64gc-unknown-none-elf/debug/dante"

kernel:
    cargo build

run *EXTRA_ARGS:
    qemu-system-riscv64 {{EXTRA_ARGS}} -M virt -m 2G -nographic -kernel {{kernel_path}}

debug: (run "-gdb tcp::1234 -S")

lldb:
    lldb {{kernel_path}} --one-line 'gdb-remote localhost:1234'
