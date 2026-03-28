// popup.js for Firefox
const b = typeof browser !== "undefined" ? browser : chrome;

const unlockScreen = document.getElementById("unlock-screen");
const mainScreen = document.getElementById("main-screen");
const passphraseInput = document.getElementById("passphrase");
const unlockBtn = document.getElementById("unlock-btn");
const unlockError = document.getElementById("unlock-error");
const searchInput = document.getElementById("search");
const listEl = document.getElementById("list");
const statusEl = document.getElementById("status");

let cachedPassphrase = null;

// --- Startup: check session cache, then GPG agent status ---
b.storage.session.get("passphrase").then((data) => {
  if (data.passphrase) {
    cachedPassphrase = data.passphrase;
    showMainScreen();
  } else {
    statusEl.innerText = "Checking GPG agent...";
    b.runtime.sendMessage({ action: "check-passphrase" }).then((response) => {
      statusEl.innerText = "";
      if (response && response.status === "ok") {
        showMainScreen();
      } else {
        showUnlockScreen();
      }
    });
  }
});

// --- Unlock screen ---
function showUnlockScreen() {
  unlockScreen.style.display = "block";
  mainScreen.style.display = "none";
  passphraseInput.focus();
}

unlockBtn.addEventListener("click", doUnlock);
passphraseInput.addEventListener("keydown", (e) => {
  if (e.key === "Enter") doUnlock();
});

function doUnlock() {
  const pw = passphraseInput.value;
  if (!pw) return;

  unlockError.innerText = "";
  unlockBtn.disabled = true;
  unlockBtn.textContent = "Unlocking...";

  b.runtime.sendMessage({ action: "unlock", passphrase: pw }).then((response) => {
    unlockBtn.disabled = false;
    unlockBtn.textContent = "Unlock";

    if (response && response.status === "ok") {
      cachedPassphrase = pw;
      b.storage.session.set({ passphrase: pw });
      passphraseInput.value = "";
      showMainScreen();
    } else {
      unlockError.innerText = (response && response.error) || "Wrong passphrase";
      passphraseInput.value = "";
      passphraseInput.focus();
    }
  });
}

// --- Main screen ---
function showMainScreen() {
  unlockScreen.style.display = "none";
  mainScreen.style.display = "block";
  searchInput.focus();
  initSearch();
}

function initSearch() {
  b.tabs.query({ active: true, currentWindow: true }).then((tabs) => {
    if (tabs.length === 0) return;
    const url = tabs[0].url || "";
    let domain = "";
    try {
      const u = new URL(url);
      domain = u.hostname.replace(/^www\./, "");
    } catch (e) {
      domain = "";
    }

    if (domain) {
      searchInput.value = domain;
      doSearch(domain);
    } else {
      statusEl.innerText = "No domain detected";
    }
  });
}

searchInput.addEventListener("input", (e) => {
  doSearch(e.target.value);
});

function doSearch(term) {
  if (!term) {
    listEl.innerHTML = "";
    statusEl.innerText = "";
    return;
  }

  statusEl.innerText = "Searching...";
  b.runtime.sendMessage({ action: "search", domain: term, passphrase: cachedPassphrase }).then((response) => {
    statusEl.innerText = "";
    if (response && response.error) {
      statusEl.innerText = "Error: " + response.error;
      return;
    }

    if (response && response.status === "ok" && response.results) {
      renderList(response.results);
    } else {
      listEl.innerHTML = "<li class='empty'>No matches found</li>";
    }
  });
}

function renderList(results) {
  listEl.innerHTML = "";
  if (results.length === 0) {
    listEl.innerHTML = "<li class='empty'>No matches found</li>";
    return;
  }

  results.forEach(([path, fields]) => {
    const li = document.createElement("li");
    li.textContent = path;
    li.addEventListener("click", () => {
      fetchAndFill(path);
    });
    listEl.appendChild(li);
  });
}

function fetchAndFill(path) {
  statusEl.innerText = "Fetching...";
  b.runtime.sendMessage({ action: "fetch", entry: path, passphrase: cachedPassphrase }).then((response) => {
    if (response && response.error) {
      statusEl.innerText = "Decrypt Error: " + response.error;
      return;
    }

    const raw = (response && response.rawEntry) || "";
    const lines = raw.split("\n");
    const password = lines[0] || "";
    let username = "";

    for (const line of lines) {
      const match = line.match(/^(username|user|login|email):\s*(.*)$/i);
      if (match) {
        username = match[2].trim();
        break;
      }
    }

    b.tabs.query({ active: true, currentWindow: true }).then((tabs) => {
      if (tabs[0] && tabs[0].id) {
        b.scripting.executeScript({
          target: { tabId: tabs[0].id },
          func: fillForm,
          args: [username, password]
        }).then(() => {
          statusEl.innerText = "Filled!";
          setTimeout(() => window.close(), 1000);
        });
      }
    });
  });
}

function fillForm(user, pass) {
  const inputs = document.querySelectorAll("input");
  let passInput = null;
  let userInput = null;

  for (const input of inputs) {
    if (input.type === "password" && input.offsetParent !== null) {
      passInput = input;
      break;
    }
  }

  if (passInput) {
    passInput.value = pass;
    passInput.dispatchEvent(new Event('input', { bubbles: true }));
    passInput.dispatchEvent(new Event('change', { bubbles: true }));

    let prev = passInput;
    while (prev = prev.previousElementSibling || (prev.parentNode && prev.parentNode.previousElementSibling)) {
      const el = prev.tagName === 'INPUT' ? prev : prev.querySelector('input');
      if (el && (el.type === 'text' || el.type === 'email')) {
        userInput = el;
        break;
      }
      if (prev.tagName === 'FORM') break;
    }

    if (!userInput) {
      for (const input of inputs) {
        if (input === passInput) break;
        if ((input.type === 'text' || input.type === 'email') && input.offsetParent !== null) {
          userInput = input;
        }
      }
    }

    if (userInput && user) {
      userInput.value = user;
      userInput.dispatchEvent(new Event('input', { bubbles: true }));
      userInput.dispatchEvent(new Event('change', { bubbles: true }));
    }
  } else {
    alert("Could not find a password field on this page.");
  }
}
