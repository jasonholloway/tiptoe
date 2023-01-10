#!/bin/bash
set -x

exec 3<>/dev/tcp/127.0.0.1/17878

{
    while read line
    do
        echo "$line" >> ~/tiptoe.log 
        
        len=${#line}
        # val=$((16#0807060504030201))
        echo "0: $(printf '%016x' $len | tac -rs ..)" | xxd -r
        echo -n "$line"
    done
} <&3 >&1 &

{
    while read -d';' marker data
    do
        if [[ $marker == '*' ]]
        then echo "$data"
        fi
    done
} <&0 >&3 &

wait
