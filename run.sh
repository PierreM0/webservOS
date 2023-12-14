GDB=${GDB:-0}
NOGRAPHIC=${NOGRAPHIC:-0}
TAP_IF=${TAP_IF:-tap0}

if [ $GDB == "0" ]
then
	gdb=""
else
	gdb="-s -S"
fi

if [ $NOGRAPHIC == "0" ]
then
	graphics="-serial stdio"
else
	graphics="-nographic"
fi


qemu-system-i386 $gdb $graphics -kernel $1 -device rtl8139,bus=pci.0,addr=4,mac=69:69:69:69:69:69

