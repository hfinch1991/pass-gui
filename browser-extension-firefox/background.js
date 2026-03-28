// background.js for Firefox
const b = typeof browser !== "undefined" ? browser : chrome;
const nativeHostName = "com.github.browserpass.native";

b.runtime.onMessage.addListener((request, sender, sendResponse) => {
  if (request.action === "check-passphrase") {
    b.runtime.sendNativeMessage(nativeHostName, { action: "check-passphrase" })
      .then(response => sendResponse(response || { status: "error", error: "No response" }))
      .catch(err => sendResponse({ status: "error", error: err.toString() }));
    return true;
  }

  if (request.action === "unlock") {
    b.runtime.sendNativeMessage(nativeHostName, { action: "unlock", passphrase: request.passphrase })
      .then(response => sendResponse(response || { status: "error", error: "No response" }))
      .catch(err => sendResponse({ status: "error", error: err.toString() }));
    return true;
  }

  if (request.action === "search") {
    b.runtime.sendNativeMessage(nativeHostName, {
      action: "search",
      domain: request.domain,
      passphrase: request.passphrase || undefined
    })
      .then(response => sendResponse(response || { status: "ok", results: [] }))
      .catch(err => sendResponse({ status: "error", error: err.toString() }));
    return true;
  }

  if (request.action === "fetch") {
    b.runtime.sendNativeMessage(nativeHostName, {
      action: "fetch",
      entry: request.entry,
      passphrase: request.passphrase || undefined
    })
      .then(response => sendResponse(response))
      .catch(err => sendResponse({ status: "error", error: err.toString() }));
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
