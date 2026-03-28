// content.js - Firefox Version
const b = typeof browser !== "undefined" ? browser : chrome;

let matchedEntries = [];
let currentDomain = window.location.hostname.replace(/^www\./, "");
let cachedPassphrase = null;

function init() {
  if (!document.querySelector('input[type="password"]')) {
    setTimeout(init, 1500);
    return;
  }
  b.storage.session.get("passphrase").then((data) => {
    cachedPassphrase = data.passphrase || null;
    b.runtime.sendMessage({ action: "search", domain: currentDomain, passphrase: cachedPassphrase }).then((response) => {
      if (response && response.status === "ok") {
        matchedEntries = response.results || [];
        injectIcons();
      }
    });
  });
}

function injectIcons() {
  const inputs = document.querySelectorAll('input[type="password"]');
  inputs.forEach(input => {
    if (!input.dataset.passGuiAttached && input.offsetParent !== null) {
      addIconToInput(input);
    }
  });
}

function addIconToInput(input) {
  input.dataset.passGuiAttached = "true";
  const host = document.createElement('div');
  host.className = 'pass-gui-host';
  host.style.cssText = 'all:initial; position:fixed; z-index:2147483647; pointer-events:none;';
  document.body.appendChild(host);
  const shadow = host.attachShadow({ mode: 'open' });

  const icon = document.createElement('div');
  icon.style.cssText = 'cursor:pointer; display:flex; align-items:center; justify-content:center; background:#fff; border-radius:4px; padding:2px; box-shadow:0 1px 4px rgba(0,0,0,0.3); border:1px solid #89b4fa; width:20px; height:20px; pointer-events:auto; transition: transform 0.1s;';
  icon.innerHTML = `<svg viewBox="0 0 24 24" fill="none" stroke="#89b4fa" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" style="width:16px;height:16px;"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"></rect><path d="M7 11V7a5 5 0 0 1 10 0v4"></path></svg>`;
  shadow.appendChild(icon);

  const updatePosition = () => {
    if (!input.isConnected || input.offsetParent === null) {
      host.style.display = 'none';
      return;
    }
    host.style.display = 'block';
    const rect = input.getBoundingClientRect();
    host.style.top = (rect.top + (rect.height - 24) / 2) + 'px';
    host.style.left = (rect.left + rect.width - 30) + 'px';
  };

  updatePosition();
  if (window.visualViewport) {
    window.visualViewport.addEventListener('resize', updatePosition);
    window.visualViewport.addEventListener('scroll', updatePosition);
  }
  window.addEventListener('resize', updatePosition);
  window.addEventListener('scroll', updatePosition, { passive: true });

  const observer = new ResizeObserver(updatePosition);
  observer.observe(input);
  observer.observe(document.body);

  icon.addEventListener('click', (e) => {
    e.preventDefault(); e.stopPropagation();
    showMenu(input);
  });
}

function showMenu(input) {
  const old = document.querySelector('.pass-gui-menu-host');
  if (old) old.remove();

  const menuHost = document.createElement('div');
  menuHost.className = 'pass-gui-menu-host';
  menuHost.style.cssText = 'all:initial; position:fixed; z-index:2147483647;';
  document.body.appendChild(menuHost);
  const shadow = menuHost.attachShadow({ mode: 'open' });

  const style = document.createElement('style');
  style.textContent = `
    .menu { background:#1e1e2e; border:1px solid #45475a; border-radius:8px; box-shadow:0 8px 32px rgba(0,0,0,0.5); padding:6px; width:280px; font-family:sans-serif; color:#cdd6f4; }
    .item { padding:10px; cursor:pointer; border-radius:6px; transition:background 0.2s; border-bottom:1px solid #313244; }
    .item:hover { background:#313244; }
    .save-item { background:#252539; border:1px dashed #89b4fa; margin-bottom:8px; color:#89b4fa; font-weight:bold; text-align:center; }
    .save-item:hover { background:#2e2e4a; }
    .title { font-weight:bold; font-size:14px; margin-bottom:2px; }
    .user { font-size:12px; color:#a6adc8; }
  `;
  shadow.appendChild(style);

  const menu = document.createElement('div');
  menu.className = 'menu';

  const saveBtn = document.createElement('div');
  saveBtn.className = 'item save-item';
  saveBtn.textContent = 'Save current credentials';
  saveBtn.onclick = () => saveCurrent(input, menuHost);
  menu.appendChild(saveBtn);

  matchedEntries.forEach(([path, fields]) => {
    const item = document.createElement('div');
    item.className = 'item';
    item.innerHTML = `<div class="title">${path.split('/').pop()}</div><div class="user">${fields.username || '(no user)'}</div>`;
    item.onclick = () => {
      fetchAndFill(path);
      menuHost.remove();
    };
    menu.appendChild(item);
  });

  shadow.appendChild(menu);

  const updateMenuPos = () => {
    if (!input.isConnected) { menuHost.remove(); return; }
    const rect = input.getBoundingClientRect();
    menuHost.style.top = (rect.bottom + 6) + 'px';
    menuHost.style.left = (rect.left) + 'px';
  };
  updateMenuPos();

  if (window.visualViewport) {
    window.visualViewport.addEventListener('resize', updateMenuPos);
    window.visualViewport.addEventListener('scroll', updateMenuPos);
  }

  const closer = (e) => {
    if(!menuHost.contains(e.target)) {
      menuHost.remove();
      document.removeEventListener('mousedown', closer);
      if (window.visualViewport) {
        window.visualViewport.removeEventListener('resize', updateMenuPos);
        window.visualViewport.removeEventListener('scroll', updateMenuPos);
      }
    }
  };
  setTimeout(() => document.addEventListener('mousedown', closer), 10);
}

function saveCurrent(input, menuHost) {
  const inputs = Array.from(document.querySelectorAll('input'));
  const pass = input.value;
  let user = "";
  const idx = inputs.indexOf(input);
  for (let j = idx - 1; j >= 0; j--) {
    if ((inputs[j].type === 'text' || inputs[j].type === 'email') && inputs[j].value) {
      user = inputs[j].value;
      break;
    }
  }
  if (!pass) { alert("Please type a password first."); return; }
  const path = prompt("Save entry as path:", `${currentDomain}/${user || 'account'}`);
  if (!path) return;
  b.runtime.sendMessage({ action: "save", path, username: user, password: pass, url: window.location.href }).then((res) => {
    if (res.status === "ok") { alert("Successfully saved!"); menuHost.remove(); init(); }
    else { alert("Error: " + res.error); }
  });
}

function fetchAndFill(path) {
  b.storage.session.get("passphrase").then((data) => {
    const pp = data.passphrase || null;
    b.runtime.sendMessage({ action: "fetch", entry: path, passphrase: pp }).then((res) => {
      if (res && res.rawEntry) {
        const lines = res.rawEntry.split('\n');
        const pass = lines[0];
        let user = "";
        for (const line of lines) {
          const m = line.match(/^(username|user|login|email):\s*(.*)$/i);
          if (m) { user = m[2].trim(); break; }
        }
        fillPage(user, pass);
      }
    });
  });
}

function fillPage(user, pass) {
  const inputs = Array.from(document.querySelectorAll('input'));
  const passInput = inputs.find(i => i.type === 'password' && i.offsetParent !== null);
  if (passInput) {
    passInput.value = pass;
    passInput.dispatchEvent(new Event('input', { bubbles: true }));
    const idx = inputs.indexOf(passInput);
    for (let j = idx - 1; j >= 0; j--) {
      if ((inputs[j].type === 'text' || inputs[j].type === 'email') && inputs[j].offsetParent !== null) {
        inputs[j].value = user;
        inputs[j].dispatchEvent(new Event('input', { bubbles: true }));
        break;
      }
    }
  }
}

init();
const globalObserver = new MutationObserver(() => injectIcons());
globalObserver.observe(document.body, { childList: true, subtree: true });
