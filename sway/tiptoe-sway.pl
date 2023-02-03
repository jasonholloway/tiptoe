#!/usr/bin/perl
use strict;
use warnings;

use Fcntl;
use Socket;

my $swaySock=$ENV{'SWAYSOCK'};

socket(SWAY, PF_UNIX, Socket::SOCK_STREAM | Socket::SOCK_CLOEXEC, 0)
    or die "Can't create socket";
connect(SWAY, sockaddr_un($swaySock))
    or die "can't connect to server!";
fcntl(SWAY, F_SETFL, fcntl(SWAY, F_GETFL, 0) | O_NONBLOCK)
    or die "fcntl problem";
SWAY->autoflush(1);


socket(TIPTOE, PF_INET, Socket::SOCK_STREAM | Socket::SOCK_CLOEXEC, getprotobyname('tcp'))
    or die "Can't create socket";
connect(TIPTOE, sockaddr_in(17878, inet_aton("127.0.0.1")))
    or die "can't connect to server!";
fcntl(TIPTOE, F_SETFL, fcntl(TIPTOE, F_GETFL, 0) | O_NONBLOCK)
    or die "fcntl problem";
TIPTOE->autoflush(1);


open(DUMP, '>', 'sway_ipc.dump')
    or die "Can't open dump file";
DUMP->autoflush(1);


STDERR->autoflush(1);


socket(LOG, PF_INET, Socket::SOCK_STREAM, getprotobyname('tcp'))
    or die "Can't create socket";
connect(LOG, sockaddr_in(17879, inet_aton("127.0.0.1")))
    or die "can't connect to server!";
LOG->autoflush(1);


my $swayInBuff = "";
my $swayOutBuff = "";

my $tiptoeInBuff = "";
my $tiptoeOutBuff = "";

tiptoeWrite('hello sway');
swayWrite(2,'["window","binding"]');

my $currCid = 0;
my $expectedCid = 0;

while (1) {
    my $workDone = 0;
    my $tiptoeInLine = "";
    my $swayInType = -1;
    my $swayInPayload = "";

    {
        my $c = sysread(TIPTOE, $tiptoeInBuff, 4096, length($tiptoeInBuff));
        if(defined($c) && $c > 0) {
            $workDone = 1;
        }
    }

    {
        my $c = sysread(SWAY, $swayInBuff, 4096, length($swayInBuff));
        if(defined($c) && $c > 0) {
            $workDone = 1;
        }
    }

    if(length($tiptoeOutBuff)) {
        my $c = syswrite(TIPTOE,$tiptoeOutBuff);
        if(defined($c) && $c > 0) {
            $tiptoeOutBuff = substr($tiptoeOutBuff, $c);
            $workDone = 1;
        }
    }

    if(length($swayOutBuff)) {
        my $c = syswrite(SWAY,$swayOutBuff);
        if(defined($c) && $c > 0) {
            print DUMP substr($swayOutBuff, 0, $c);
            $swayOutBuff = substr($swayOutBuff, $c);
            $workDone = 1;
        }
    }

    if ($tiptoeInBuff =~ /^(.*?)\n(.*)$/) {
        $tiptoeInLine = $1;
        print LOG "\e[1;31m< " . $tiptoeInLine . "\e[1;0m\n";
        
        $tiptoeInBuff = $2;
        $workDone = 1;
    }

    # below relies on all messages being within buffer - aargh!
    if ($swayInBuff =~ /^i3-ipc(.{4})(.{4})(.*)/) {
        my $l = unpack('V', $1);
        $swayInType = unpack('V', $2);
        $swayInPayload = substr($3, 0, $l);
        $swayInBuff = substr($3, $l);
        $workDone = 1;
    }

    {
        if ($swayInType == 0x80000003) {
            if ($swayInPayload =~ /"change":\W*"focus"/) {
                my $cid = 0;
                my $appId = "";
                my $pid = 0;
                my $name = "";

                if ($swayInPayload =~ /"container":\W*\{\W*"id": (\d+)/) {
                    $cid = $1 + 0;
                }

                if ($swayInPayload =~ /"app_id":\W*"([^"]+)"/) {
                    $appId = $1;
                }

                if ($swayInPayload =~ /"pid":\W*(\d+)/) {
                    $pid = $1 + 0;
                }

                if ($swayInPayload =~ /"name":\W*"([^"]+)"/) {
                    $name = $1;
                }

                if ($cid != $currCid) {
                    if ($cid != $expectedCid) {
                        tiptoeWrite("stepped $currCid`` $cid``");
                    }
                    $currCid = $cid;
                }
            }
        }

        if ($swayInType == 0x80000005) {
            if ($swayInPayload =~ /"change":\W*"run"/) {
                if ($swayInPayload =~ /"symbols":\W*\[\W*"grave"\W*\]/) {
                    tiptoeWrite("reverse");
                }
                if ($swayInPayload =~ /"symbols":\W*\[\W*"Tab"\W*\]/) {
                    tiptoeWrite("hop");
                }
            }
        }

        if ($tiptoeInLine =~ /^goto (\d+)`/) {
            my $cid = $1;
            $expectedCid=$cid;
            swayWrite(0, "[con_id=$cid] focus");
        }
    }

    if(!$workDone) {
        # sleep 1;
        select(undef, undef, undef, 0.05);
    }
}

close(SWAY);
close(TIPTOE);
close(DUMP);


sub swayWrite {
    my ($type,$payload) = @_;
    print LOG "\e[1;33m> " . $type . " " . $payload . "\e[1;0m\n\n";
    $swayOutBuff .= "i3-ipc" . pack("V",length($payload)) . pack("V",$type). $payload;
}

sub tiptoeWrite {
    my ($line) = @_;
    print LOG "\e[1;32m> " . $line . "\e[1;0m\n";
    $tiptoeOutBuff .= $line . "\n";
}
