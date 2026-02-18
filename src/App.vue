<script setup lang="ts">
import { onMounted, ref } from "vue";
import { usePasswordStore } from "./stores/password";
import { useSetupStore } from "./stores/setup";
import Toolbar from "./components/Toolbar.vue";
import SearchBar from "./components/SearchBar.vue";
import TreeView from "./components/TreeView.vue";
import DetailPanel from "./components/DetailPanel.vue";
import SetupWizard from "./components/SetupWizard.vue";

const store = usePasswordStore();
const setupStore = useSetupStore();

const showWizard = ref(false);
const checkingSetup = ref(true);

async function onWizardComplete() {
  showWizard.value = false;
  await store.loadTree();
}

onMounted(async () => {
  await setupStore.checkStatus();
  if (setupStore.setupStatus?.needs_setup) {
    showWizard.value = true;
  } else {
    store.loadTree();
  }
  checkingSetup.value = false;
});
</script>

<template>
  <div v-if="checkingSetup" class="app loading-screen">
    <div class="status">Loading...</div>
  </div>
  <SetupWizard v-else-if="showWizard" @complete="onWizardComplete" />
  <div v-else class="app">
    <header class="app-header">
      <Toolbar />
    </header>
    <div class="app-body">
      <aside class="sidebar">
        <SearchBar />
        <div class="tree-container">
          <div v-if="store.loading" class="status">Loading...</div>
          <div v-else-if="store.error" class="status error">{{ store.error }}</div>
          <TreeView v-else :nodes="store.filteredTree" :depth="0" />
        </div>
      </aside>
      <main class="detail">
        <DetailPanel />
      </main>
    </div>
  </div>
</template>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

:root {
  --bg: #1e1e2e;
  --bg-surface: #252536;
  --bg-hover: #2e2e42;
  --bg-active: #383850;
  --text: #cdd6f4;
  --text-dim: #888;
  --accent: #89b4fa;
  --accent-hover: #74a8fc;
  --border: #383850;
  --danger: #f38ba8;
  --success: #a6e3a1;
  --radius: 6px;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
  background: var(--bg);
  color: var(--text);
  font-size: 13px;
}

.app {
  display: flex;
  flex-direction: column;
  height: 100vh;
}

.app-header {
  border-bottom: 1px solid var(--border);
  background: var(--bg-surface);
}

.app-body {
  display: flex;
  flex: 1;
  overflow: hidden;
}

.sidebar {
  width: 300px;
  min-width: 200px;
  border-right: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  background: var(--bg-surface);
}

.tree-container {
  flex: 1;
  overflow-y: auto;
  padding: 4px 0;
}

.detail {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
}

.status {
  padding: 16px;
  color: var(--text-dim);
}

.status.error {
  color: var(--danger);
}

.loading-screen {
  display: flex;
  align-items: center;
  justify-content: center;
}

button {
  background: var(--bg-hover);
  color: var(--text);
  border: 1px solid var(--border);
  padding: 5px 10px;
  border-radius: var(--radius);
  cursor: pointer;
  font-size: 12px;
  white-space: nowrap;
}

button:hover {
  background: var(--bg-active);
}

button.primary {
  background: var(--accent);
  color: #1e1e2e;
  border-color: var(--accent);
}

button.primary:hover {
  background: var(--accent-hover);
}

button.danger {
  background: var(--danger);
  color: #1e1e2e;
  border-color: var(--danger);
}

input, textarea, select {
  background: var(--bg);
  color: var(--text);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 6px 8px;
  font-size: 13px;
  font-family: inherit;
}

input:focus, textarea:focus, select:focus {
  outline: none;
  border-color: var(--accent);
}

.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}

.modal {
  background: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 20px;
  min-width: 400px;
  max-width: 500px;
}

.modal h3 {
  margin-bottom: 12px;
  font-size: 15px;
}

.modal .field {
  margin-bottom: 12px;
}

.modal .field label {
  display: block;
  margin-bottom: 4px;
  font-size: 12px;
  color: var(--text-dim);
}

.modal .field input,
.modal .field textarea {
  width: 100%;
}

.modal .actions {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
  margin-top: 16px;
}

.modal .checkbox-row {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 8px;
}
</style>
