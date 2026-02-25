<script setup lang="ts">
import { onMounted, ref, computed } from "vue";
import { useSetupStore } from "../stores/setup";

const setup = useSetupStore();
const copiedCmd = ref(false);

const platform = computed(() => {
  const ua = navigator.userAgent.toLowerCase();
  if (ua.includes("mac")) return "macos";
  if (ua.includes("win")) return "windows";
  return "linux";
});

const installCommands = computed(() => {
  switch (platform.value) {
    case "macos":
      return { label: "Homebrew", cmd: "brew install gnupg git" };
    case "linux":
      return {
        label: "Package Manager",
        commands: [
          { label: "Ubuntu/Debian", cmd: "sudo apt install gnupg git" },
          { label: "Fedora", cmd: "sudo dnf install gnupg2 git" },
          { label: "Arch", cmd: "sudo pacman -S gnupg git" },
        ],
      };
    case "windows":
      return {
        label: "Manual Install",
        links: [
          { label: "Gpg4win", url: "https://www.gpg4win.org/" },
          { label: "Git for Windows", url: "https://git-scm.com/download/win" },
        ],
      };
    default:
      return null;
  }
});

async function copyCommand(cmd: string) {
  await navigator.clipboard.writeText(cmd);
  copiedCmd.value = true;
  setTimeout(() => { copiedCmd.value = false; }, 1500);
}

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
      Pass GUI requires <strong>gpg</strong> and
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
      <!-- macOS -->
      <template v-if="platform === 'macos'">
        <button
          class="primary"
          :disabled="setup.installLoading"
          @click="setup.installDeps()"
        >
          {{ setup.installLoading ? "Installing..." : "Install via Homebrew" }}
        </button>
        <div v-if="setup.installError" class="error">{{ setup.installError }}</div>
      </template>

      <!-- Linux -->
      <template v-else-if="platform === 'linux'">
        <p class="install-hint">Install using your package manager:</p>
        <div v-for="c in (installCommands as any).commands" :key="c.label" class="cmd-row">
          <span class="cmd-label">{{ c.label }}:</span>
          <code class="cmd-code">{{ c.cmd }}</code>
          <button class="copy-btn" @click="copyCommand(c.cmd)">Copy</button>
        </div>
        <span v-if="copiedCmd" class="copied-msg">Copied!</span>
        <p class="install-hint refresh-hint">After installing, click Refresh to re-check.</p>
        <button @click="setup.checkStatus()">Refresh</button>
      </template>

      <!-- Windows -->
      <template v-else-if="platform === 'windows'">
        <p class="install-hint">Please download and install:</p>
        <div v-for="link in (installCommands as any).links" :key="link.label" class="link-row">
          <a :href="link.url" target="_blank" rel="noopener">{{ link.label }}</a>
        </div>
        <p class="install-hint refresh-hint">After installing, click Refresh to re-check.</p>
        <button @click="setup.checkStatus()">Refresh</button>
      </template>
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
.install-hint {
  color: var(--text-dim);
  font-size: 12px;
  margin-bottom: 8px;
}
.refresh-hint {
  margin-top: 8px;
}
.cmd-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 6px;
}
.cmd-label {
  font-size: 12px;
  color: var(--text-dim);
  min-width: 90px;
}
.cmd-code {
  background: var(--bg);
  padding: 4px 8px;
  border-radius: 3px;
  font-size: 12px;
  flex: 1;
}
.copy-btn {
  padding: 2px 8px;
  font-size: 11px;
}
.link-row {
  margin-bottom: 6px;
}
.link-row a {
  color: var(--accent);
  text-decoration: none;
  font-size: 13px;
}
.link-row a:hover {
  text-decoration: underline;
}
.copied-msg {
  color: var(--success);
  font-size: 12px;
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
