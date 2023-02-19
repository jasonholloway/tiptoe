installMediatorApp: firefox/mediator/tiptoe-firefox-mediate.pl
	ln -s -f /usr/local/src/tiptoe/firefox/mediator/tiptoe-firefox-mediate.pl /usr/local/bin/tiptoe-firefox-mediate

installMediatorFirefox: firefox/mediator/tiptoe_firefox_mediate.json
	mkdir -p ~/.mozilla/native-messaging-hosts
	cp firefox/mediator/tiptoe_firefox_mediate.json ~/.mozilla/native-messaging-hosts/

installServer: server/target/debug/tiptoe
	cp server/target/debug/tiptoe /usr/local/bin/tiptoe

install: installMediatorApp installMediatorFirefox installServer



server/target/debug/tiptoe: $(wildcard server/src/*.rs)
	cd server && cargo build

build: server/target/debug/tiptoe


.PHONY: build install
