installMediatorApp: firefox/mediator/tiptoe-firefox-mediate.pl
	ln -s -f /usr/local/src/tiptoe/firefox/mediator/tiptoe-firefox-mediate.pl /usr/local/bin/tiptoe-firefox-mediate

installMediatorFirefox: firefox/mediator/tiptoe_firefox_mediate.json
	mkdir -p ~/.mozilla/native-messaging-hosts
	cp firefox/mediator/tiptoe_firefox_mediate.json ~/.mozilla/native-messaging-hosts/

install: installMediatorApp installMediatorFirefox

.PHONY: install
