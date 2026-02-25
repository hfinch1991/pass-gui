// popup.js

const searchInput = document.getElementById("search");
const listEl = document.getElementById("list");
const statusEl = document.getElementById("status");

// 1. Get current tab URL and search
chrome.tabs.query({ active: true, currentWindow: true }, (tabs) => {
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
  chrome.runtime.sendMessage({ action: "search", domain: term }, (response) => {
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
  chrome.runtime.sendMessage({ action: "fetch", entry: path }, (response) => {
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
    
    chrome.tabs.query({ active: true, currentWindow: true }, (tabs) => {
      if (tabs[0] && tabs[0].id) {
        chrome.scripting.executeScript({
          target: { tabId: tabs[0].id },
          func: fillForm,
          args: [username, password]
        }, () => {
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