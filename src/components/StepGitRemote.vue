<script setup lang="ts">
import { useSetupStore } from "../stores/setup";

const setup = useSetupStore();
</script>

<template>
  <div class="step">
    <h3>Git Sync (Optional)</h3>
    <p class="step-desc">
      Set up git to sync your password store with a remote repository.
      You can skip this step and configure it later from Settings.
    </p>

    <!-- Show current git status if store has git -->
    <div v-if="setup.storeInfo?.has_git && setup.storeInfo?.git_remote && !setup.gitConfigured" class="current-info">
      <div class="info-header">
        <span class="ok">&#x2713;</span>
        <span>Git is already configured</span>
      </div>
      <div class="detail-row">
        <span class="label">Remote:</span>
        <code>{{ setup.storeInfo.git_remote }}</code>
      </div>
    </div>

    <div class="checkbox-row">
      <input
        id="git-enable"
        v-model="setup.gitEnabled"
        type="checkbox"
      />
      <label for="git-enable">
        {{ setup.storeInfo?.has_git ? "Change git remote" : "Enable git sync" }}
      </label>
    </div>

    <template v-if="setup.gitEnabled">
      <div class="field">
        <label>Remote URL</label>
        <input
          v-model="setup.remoteUrl"
          type="text"
          placeholder="git@github.com:user/pass-store.git"
        />
      </div>

      <button
        class="primary"
        :disabled="setup.gitLoading || !setup.remoteUrl.trim()"
        @click="setup.setupGit()"
      >
        {{ setup.gitLoading ? "Configuring..." : "Configure Git" }}
      </button>

      <div v-if="setup.gitConfigured" class="success">
        Git configured successfully.
      </div>
      <div v-if="setup.gitError" class="error">{{ setup.gitError }}</div>
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
.current-info {
  background: var(--bg);
  border-radius: var(--radius);
  padding: 12px;
}
.info-header {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 600;
  margin-bottom: 8px;
}
.detail-row {
  display: flex;
  gap: 8px;
  font-size: 13px;
  align-items: center;
}
.detail-row .label {
  color: var(--text-dim);
}
code {
  background: var(--bg-surface);
  padding: 2px 6px;
  border-radius: 3px;
  font-size: 12px;
}
.ok {
  color: var(--success);
  font-weight: bold;
}
.checkbox-row {
  display: flex;
  align-items: center;
  gap: 8px;
}
.checkbox-row label {
  font-size: 13px;
  cursor: pointer;
}
.field {
  margin-bottom: 4px;
}
.field label {
  display: block;
  margin-bottom: 4px;
  font-size: 12px;
  color: var(--text-dim);
}
.field input {
  width: 100%;
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
