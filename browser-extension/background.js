// background.js

chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
  if (request.action === "check-passphrase") {
    chrome.runtime.sendNativeMessage("com.github.browserpass.native", { action: "check-passphrase" }, (response) => {
      if (chrome.runtime.lastError) sendResponse({ status: "error", error: chrome.runtime.lastError.message });
      else sendResponse(response || { status: "error", error: "No response" });
    });
    return true;
  }

  if (request.action === "unlock") {
    chrome.runtime.sendNativeMessage("com.github.browserpass.native", { action: "unlock", passphrase: request.passphrase }, (response) => {
      if (chrome.runtime.lastError) sendResponse({ status: "error", error: chrome.runtime.lastError.message });
      else sendResponse(response || { status: "error", error: "No response" });
    });
    return true;
  }

  if (request.action === "search") {
    chrome.runtime.sendNativeMessage("com.github.browserpass.native", {
      action: "search",
      domain: request.domain,
      passphrase: request.passphrase || undefined
    }, (response) => {
      if (chrome.runtime.lastError) sendResponse({ status: "error", error: chrome.runtime.lastError.message });
      else sendResponse(response || { status: "ok", results: [] });
    });
    return true;
  }

  if (request.action === "fetch") {
    chrome.runtime.sendNativeMessage("com.github.browserpass.native", {
      action: "fetch",
      entry: request.entry,
      passphrase: request.passphrase || undefined
    }, (response) => {
      if (chrome.runtime.lastError) sendResponse({ status: "error", error: chrome.runtime.lastError.message });
      else sendResponse(response);
    });
    return true;
  }

  if (request.action === "save") {
    chrome.runtime.sendNativeMessage("com.github.browserpass.native", {
      action: "save",
      path: request.path,
      username: request.username,
      password: request.password,
      url: request.url
    }, (response) => {
      if (chrome.runtime.lastError) sendResponse({ status: "error", error: chrome.runtime.lastError.message });
      else sendResponse(response);
    });
    return true;
  }

  sendResponse({ status: "error", error: "Unknown action" });
  return false;
});
