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


# print TIPTOE "hello sway\n";



my $swayInBuff = "";
my $swayOutBuff = "";

my $tiptoeInBuff = "";
my $tiptoeOutBuff = "";

# $tiptoeOutBuff .= "hello sway\n";
# $swayOutBuff .= "i3-ipc" . pack("V",19) . pack("V",2). '["window","binding"]';


tiptoeWrite('hello sway');
swayWrite(2,'["window","binding"]');

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
            $swayOutBuff = substr($swayOutBuff, $c);
            $workDone = 1;
        }
    }

    if ($tiptoeInBuff =~ /^(.*?)\n(.*)$/) {
        $tiptoeInLine = $1;
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

                if ($cid > 0) {
                    tiptoeWrite("visited $cid`$appId`$pid");
                }
            }
        }

        if ($swayInType == 0x80000005) {
            if ($swayInPayload =~ /"change":\W*"run"/) {
                if ($swayInPayload =~ /"symbols":\W*\[\W*"grave"\W*\]/) {
                    tiptoeWrite("reverse");
                }
            }
        }

        if ($tiptoeInLine =~ /^revisit (\d+)`/) {
            my $cid = $1;
            swayWrite(0, "[cond_id=$cid] focus");
        }
    }

    if(!$workDone) {
        # sleep 1;
        select(undef, undef, undef, 0.05);
    }
}

close(SWAY);
close(TIPTOE);


sub swayWrite {
    my ($type,$payload) = @_;
    $swayOutBuff .= "i3-ipc" . pack("V",length($payload)) . pack("V",$type). $payload;
}

sub tiptoeWrite {
    my ($line) = @_;
    $tiptoeOutBuff .= $line . "\n";
}
