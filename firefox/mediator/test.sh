#!/bin/bash
exec 3<>/dev/tcp/vm/17879
cat >&3
