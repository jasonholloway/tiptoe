#!/usr/bin/perl
use strict;
use warnings;

use Fcntl;
use Socket;

socket(LOG, PF_INET, Socket::SOCK_STREAM, getprotobyname('tcp'))
    or die "Can't create socket";
connect(LOG, sockaddr_in(17879, inet_aton("100.110.110.38")))
    or die "can't connect to server!";
LOG->autoflush(1);


fcntl(STDIN, F_SETFL, fcntl(STDIN, F_GETFL, 0) | O_NONBLOCK)
    or die "fcntl problem";


socket(SERVER, PF_INET, Socket::SOCK_STREAM | Socket::SOCK_CLOEXEC, getprotobyname('tcp'))
    or die "Can't create socket";
connect(SERVER, sockaddr_in(17878, inet_aton("127.0.0.1")))
    or die "can't connect to server!";
fcntl(SERVER, F_SETFL, fcntl(SERVER, F_GETFL, 0) | O_NONBLOCK)
    or die "fcntl problem";
SERVER->autoflush(1);

STDOUT->autoflush(1);


print LOG "connecting...\n";

print SERVER "hello ff\n";

my $stdinBuffer;
my $serverBuffer;
my $serverBufferOffset = 0;

while (1) {
    my $workDone = 0;
    
    if(defined(sysread(STDIN, $stdinBuffer, 4))) {
        $workDone = 1;

        my $c = unpack("V",$stdinBuffer);
        if(defined($c) && defined(sysread(STDIN, $stdinBuffer, $c))) {
            $stdinBuffer =~ s/(^")|("$)//g;
            print SERVER "$stdinBuffer\n";
            print LOG "ff>tiptoe $c $stdinBuffer\n";
        }
    }

    {
        my $c = sysread(SERVER, $serverBuffer, 128, $serverBufferOffset);
        if(defined($c)) {
            if ($serverBuffer =~ /^(.*?)\n(.*)$/) {
                my $s = '"' . $1 . '"';
                my $l = length($s);
                print STDOUT pack("V", $l);
                print STDOUT $s;
                print LOG "tiptoe>ff $l $s\n";

                $serverBuffer = $2;
                $serverBufferOffset = length($2);
            }
            else {
                $serverBufferOffset += $c;
            }
            $workDone = 1;
        }
    }

    if(!$workDone) {
        select(undef, undef, undef, 0.05);
    }
}

close(SERVER)
