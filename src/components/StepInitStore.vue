<script setup lang="ts">
import { useSetupStore } from "../stores/setup";

const setup = useSetupStore();
</script>

<template>
  <div class="step">
    <h3>Password Store</h3>

    <!-- Existing store found: ask user to confirm or init new -->
    <template v-if="setup.setupStatus?.store_exists">
      <div class="found-box">
        <div class="found-header">
          <span class="ok">&#x2713;</span>
          <span>Found existing password store</span>
        </div>
        <div class="found-details">
          <div class="detail-row">
            <span class="label">Path:</span>
            <code>{{ setup.storeInfo?.store_path || "~/.password-store" }}</code>
          </div>
          <div v-if="setup.storeInfo?.gpg_id" class="detail-row">
            <span class="label">GPG ID:</span>
            <code>{{ setup.storeInfo.gpg_id }}</code>
          </div>
          <div class="detail-row">
            <span class="label">Git:</span>
            <span>{{ setup.storeInfo?.has_git ? "Enabled" : "Not initialized" }}</span>
          </div>
          <div v-if="setup.storeInfo?.git_remote" class="detail-row">
            <span class="label">Remote:</span>
            <code>{{ setup.storeInfo.git_remote }}</code>
          </div>
        </div>
      </div>

      <div v-if="!setup.useExistingStore" class="confirm-actions">
        <button class="primary" @click="setup.confirmExistingStore()">
          Use This Store
        </button>
        <p class="hint">Or go back to select a different GPG key and re-initialize.</p>
      </div>
      <div v-else class="success">
        Using existing password store.
      </div>
    </template>

    <!-- No store: init new -->
    <template v-else-if="setup.storeInitialized">
      <div class="success">Password store initialized successfully.</div>
    </template>

    <template v-else>
      <p class="step-desc">
        Initialize a new password store at <code>~/.password-store/</code> using
        the selected GPG key.
      </p>

      <div class="key-info">
        <span class="label">GPG Key:</span>
        <span>{{ setup.selectedKeyId }}</span>
      </div>

      <button
        class="primary"
        :disabled="setup.initLoading || !setup.selectedKeyId"
        @click="setup.initStore()"
      >
        {{ setup.initLoading ? "Initializing..." : "Initialize Store" }}
      </button>

      <div v-if="setup.initError" class="error">{{ setup.initError }}</div>
    </template>
  </div>
</template>

<style scoped>
.step {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.step-desc {
  color: var(--text-dim);
  font-size: 13px;
  line-height: 1.5;
}
code {
  background: var(--bg);
  padding: 2px 6px;
  border-radius: 3px;
  font-size: 12px;
}
.found-box {
  background: var(--bg);
  border-radius: var(--radius);
  padding: 12px;
}
.found-header {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 600;
  margin-bottom: 10px;
}
.found-details {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding-left: 4px;
}
.detail-row {
  display: flex;
  gap: 8px;
  font-size: 13px;
  align-items: center;
}
.detail-row .label {
  color: var(--text-dim);
  min-width: 60px;
}
.confirm-actions {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.hint {
  color: var(--text-dim);
  font-size: 12px;
}
.key-info {
  display: flex;
  gap: 8px;
  padding: 8px 12px;
  background: var(--bg);
  border-radius: var(--radius);
  font-size: 13px;
}
.key-info .label {
  color: var(--text-dim);
}
.ok {
  color: var(--success);
  font-weight: bold;
}
.success {
  color: var(--success);
  font-size: 13px;
}
.error {
  color: var(--danger);
  font-size: 12px;
}
</style>
