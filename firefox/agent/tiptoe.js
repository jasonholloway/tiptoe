
const port = browser.runtime.connectNative('tiptoe_firefox_mediate');
port.onDisconnect.addListener(p => {
    if(p.error) console.error('Disconnected with error:', p.error);
});


let tabMask = null;

browser.tabs.onActivated.addListener(({tabId,previousTabId,windowId}) => {
    console.log('activated', tabId);

    if(tabId != tabMask) {
        const m = `stepped ${windowId}/${previousTabId} ${windowId}/${tabId}`;
        port.postMessage(m);
        console.log('posted', m);
        tabMask = tabId;
    }
});

port.onMessage.addListener(m => {
    console.log("received", m);
    
    if(typeof m === "string") {
        const matched = m.match(/^goto (\d+)\/(\d+)/);
        if(matched) {
            const [,rawWindowId,rawTabId] = matched;
            const windowId = parseInt(rawWindowId);
            const tabId = parseInt(rawTabId);

            console.log("switch to", windowId, tabId);

            tabMask = tabId;
            browser.tabs.update(tabId, { active:true });
        }
    }
});

