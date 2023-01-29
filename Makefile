installMediatorApp: browser/mediator/mediate.sh
	ln -s -f /usr/local/src/tiptoe/browser/mediator/mediate.pl /usr/local/bin/tiptoe-browser-mediate

installMediatorFirefox: browser/mediator/firefox/tiptoe_mediator.json
	mkdir -p ~/.mozilla/native-messaging-hosts
	cp browser/mediator/firefox/tiptoe_mediator.json ~/.mozilla/native-messaging-hosts/

install: installMediatorApp installMediatorFirefox

.PHONY: install
