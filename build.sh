set -xe
cargo build
rm -fr isodir
mkdir -p isodir/boot/grub
cp target/target/debug/webserv_os isodir/boot/myos.bin
cp grub.cfg isodir/boot/grub/grub.cfg
grub2-mkrescue -o myiso.iso isodir
