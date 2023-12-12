gdb="-s -S"
gdb=""

qemu-system-i386 $gdb -serial stdio -kernel $1 
