
const port = browser.runtime.connectNative('tiptoe_mediator');
port.onDisconnect.addListener(p => {
    if(p.error) console.error(p.error);
});

port.onMessage.addListener(m => {
    console.log('received', m);
});

browser.tabs.onActivated.addListener(({tabId,previousTabId,windowId}) => {
    console.log('activated', tabId);

    const m = `;* selected firefox/${windowId}/${tabId};`;
    console.log('posting', m);

    port.postMessage(m);
    console.log('posted', m);
});
