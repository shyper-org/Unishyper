#!/usr/bin/env bash

for value in test_inject_thread \
            shell_thread \
            input_thread \
            release \
            open \
            close \
            read \
            write \
            print_dir \

do
    echo "injecting $value"
    trap '' 2
    make emu PI=$value
    trap 2
done