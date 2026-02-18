<script setup lang="ts">
import { ref, computed } from "vue";
import { usePasswordStore } from "../stores/password";

const store = usePasswordStore();
const copied = ref(false);

const firstLine = computed(() => {
  if (!store.detailContent) return null;
  return store.detailContent.split("\n")[0];
});

async function copyToClipboard(text: string) {
  await navigator.clipboard.writeText(text);
  copied.value = true;
  setTimeout(() => { copied.value = false; }, 1500);
}

async function copyPassword() {
  if (firstLine.value) {
    await copyToClipboard(firstLine.value);
  }
}

async function copyAll() {
  if (store.detailContent) {
    await copyToClipboard(store.detailContent);
  }
}
</script>

<template>
  <div v-if="!store.selectedPath" class="empty">
    <p>Select a password entry to view its contents.</p>
  </div>
  <div v-else class="detail-panel">
    <div class="detail-header">
      <h3>{{ store.selectedPath }}</h3>
      <div class="detail-actions">
        <button @click="copyPassword" :disabled="!firstLine" title="Copy password (first line)">
          Copy Password
        </button>
        <button @click="copyAll" :disabled="!store.detailContent" title="Copy all content">
          Copy All
        </button>
        <span v-if="copied" class="copied-msg">Copied!</span>
      </div>
    </div>
    <div v-if="store.detailLoading" class="status">Decrypting...</div>
    <pre v-else class="detail-content">{{ store.detailContent }}</pre>
  </div>
</template>

<style scoped>
.empty {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--text-dim);
}

.detail-panel {
  height: 100%;
}

.detail-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
  gap: 8px;
}

.detail-header h3 {
  font-size: 14px;
  font-weight: 600;
  word-break: break-all;
}

.detail-actions {
  display: flex;
  gap: 6px;
  flex-shrink: 0;
}

.detail-content {
  background: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 12px;
  font-family: "SF Mono", Menlo, Monaco, monospace;
  font-size: 13px;
  white-space: pre-wrap;
  word-break: break-all;
  line-height: 1.5;
}

.status {
  color: var(--text-dim);
}

.copied-msg {
  color: var(--success);
  font-size: 12px;
}
</style>
