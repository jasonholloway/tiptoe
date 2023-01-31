#!/bin/bash

{
    set -x
    
    exec 3<>/dev/tcp/127.0.0.1/17878

    cat <&3 >&1 &

    {
        echo "hello ff browser"

        while read -d';' line
        do echo "${line};"
        done
    } <&0 >&3 &

    wait
} 2>~/tiptoe-mediator.log
