// background.js for Firefox
const b = typeof browser !== "undefined" ? browser : chrome;
const nativeHostName = "com.github.browserpass.native";

b.runtime.onMessage.addListener((request, sender, sendResponse) => {
  console.log("Firefox Background: action =", request.action);

  if (request.action === "search") {
    b.runtime.sendNativeMessage(nativeHostName, { action: "search", domain: request.domain })
      .then(response => {
        sendResponse(response || { status: "ok", results: [] });
      })
      .catch(err => {
        console.error("Native Messaging Error:", err);
        sendResponse({ status: "error", error: err.toString() });
      });
    return true; // Keep channel open
  }

  if (request.action === "fetch") {
    b.runtime.sendNativeMessage(nativeHostName, { action: "fetch", entry: request.entry })
      .then(response => {
        sendResponse(response);
      })
      .catch(err => {
        sendResponse({ status: "error", error: err.toString() });
      });
    return true;
  }

  if (request.action === "save") {
    b.runtime.sendNativeMessage(nativeHostName, { 
      action: "save", 
      path: request.path, 
      username: request.username, 
      password: request.password,
      url: request.url
    })
    .then(response => sendResponse(response))
    .catch(err => sendResponse({ status: "error", error: err.toString() }));
    return true;
  }

  return false;
});