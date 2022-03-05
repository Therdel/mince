#!/usr/bin/env bash
# source: https://aixxe.net/2016/09/shared-library-injection

process=hl2_linux
pid=$(pidof $process)
libraryPath=$(realpath "../target/i686-unknown-linux-gnu/debug/libmince.so")
library=$(basename $libraryPath)

# disable ptrace scope for ejection permissions
echo "disabling ptrace_scope for ejection permissions"
echo 0 | sudo tee /proc/sys/kernel/yama/ptrace_scope

echo "Process: $process (pid: $(pidof $process))"
echo "Library: $library"

if grep -q $libraryPath /proc/$pid/maps ; then
    echo "$library is loaded. Ejecting"
  sudo gdb -n -q -batch \
    -ex "attach $(pidof $process)" \
    -ex "set \$dlopen = (void*(*)(char*, int)) dlopen" \
    -ex "set \$dlclose = (int(*)(void*)) dlclose" \
    -ex "set \$library = \$dlopen(\"$libraryPath\", 6)" \
    -ex "call \$dlclose(\$library)" \
    -ex "call \$dlclose(\$library)" \
    -ex "detach" \
    -ex "quit"
else
    echo "$library is not loaded"
fi
