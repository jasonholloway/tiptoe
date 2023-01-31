
const port = browser.runtime.connectNative('tiptoe_mediator');
port.onDisconnect.addListener(p => {
    if(p.error) console.error(p.error);
});


let currTabId = null;

browser.tabs.onActivated.addListener(({tabId,previousTabId,windowId}) => {
    console.log('activated', tabId);

    if(currTabId != tabId) {
        const m = `stepped ${windowId}/${previousTabId} ${windowId}/${tabId}`;
        port.postMessage(m);
        console.log('posted', m);
        currTabId = tabId;
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

            currTabId = tabId;
            browser.tabs.update(tabId, { active:true });
        }
    }
});

