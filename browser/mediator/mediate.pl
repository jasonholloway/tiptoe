#!/usr/bin/perl
use Socket;

open(LOG, ">>", "/home/jason/tiptoe-mediator.log")
    or die "Can't open log file for writing";

socket(SERVER, PF_INET, SOCK_STREAM, getprotobyname('tcp'))
    or die "Can't create socket";

connect(SERVER, sockaddr_in(17878, inet_aton("127.0.0.1")))
    or die "can't connect to server!";

SERVER->autoflush(1);
LOG->autoflush(1);

print LOG "connecting...\n";

print SERVER "hello ff\n";

while (read(STDIN, $buffer, 4) > 1) {
    $c = unpack("V",$buffer);
    read(STDIN, $buffer, $c);

    $buffer =~ s/(^")|("$)//g;

    print SERVER "$buffer\n";

    print LOG "read $c $buffer\n";
}

close(SERVER)
