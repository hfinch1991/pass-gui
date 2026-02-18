<script setup lang="ts">
import { onMounted } from "vue";
import { useSetupStore } from "../stores/setup";

const setup = useSetupStore();

onMounted(() => {
  if (!setup.setupStatus) {
    setup.checkStatus();
  }
});
</script>

<template>
  <div class="step">
    <h3>Check Dependencies</h3>
    <p class="step-desc">
      Pass GUI requires <strong>pass</strong>, <strong>gpg</strong>, and
      <strong>git</strong> to be installed.
    </p>

    <div v-if="setup.depsLoading" class="status">Checking dependencies...</div>

    <div v-else-if="setup.setupStatus" class="dep-list">
      <div
        v-for="dep in setup.setupStatus.dependencies"
        :key="dep.name"
        class="dep-row"
      >
        <span :class="dep.installed ? 'ok' : 'missing'">
          {{ dep.installed ? "\u2713" : "\u2717" }}
        </span>
        <span class="dep-name">{{ dep.name }}</span>
        <span v-if="dep.version" class="dep-version">{{ dep.version }}</span>
        <span v-else class="dep-version missing">not found</span>
      </div>
    </div>

    <div v-if="setup.depsError" class="error">{{ setup.depsError }}</div>

    <div v-if="setup.setupStatus && !setup.setupStatus.dependencies_ok" class="install-section">
      <button
        class="primary"
        :disabled="setup.installLoading"
        @click="setup.installDeps()"
      >
        {{ setup.installLoading ? "Installing..." : "Install via Homebrew" }}
      </button>
      <div v-if="setup.installError" class="error">{{ setup.installError }}</div>
    </div>

    <div v-if="setup.setupStatus?.dependencies_ok" class="success">
      All dependencies installed.
    </div>
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
.dep-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.dep-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: var(--bg);
  border-radius: var(--radius);
}
.dep-name {
  font-weight: 600;
  min-width: 50px;
}
.dep-version {
  color: var(--text-dim);
  font-size: 12px;
}
.dep-version.missing {
  color: var(--danger);
}
.ok {
  color: var(--success);
  font-weight: bold;
}
.missing {
  color: var(--danger);
  font-weight: bold;
}
.install-section {
  padding-top: 4px;
}
.error {
  color: var(--danger);
  font-size: 12px;
}
.success {
  color: var(--success);
  font-size: 13px;
}
.status {
  color: var(--text-dim);
}
</style>
