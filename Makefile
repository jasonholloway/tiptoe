installMediatorApp: firefox/mediator/mediate.pl
	ln -s -f /usr/local/src/tiptoe/firefox/mediator/mediate.pl /usr/local/bin/tiptoe-firefox-mediate

installMediatorFirefox: firefox/mediator/tiptoe_mediator.json
	mkdir -p ~/.mozilla/native-messaging-hosts
	cp firefox/mediator/tiptoe_mediator.json ~/.mozilla/native-messaging-hosts/

install: installMediatorApp installMediatorFirefox

.PHONY: install
