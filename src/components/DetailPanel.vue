<script setup lang="ts">
import { ref, computed } from "vue";
import { usePasswordStore } from "../stores/password";

const store = usePasswordStore();
const copied = ref("");
const showPassword = ref(false);

const firstLine = computed(() => {
  if (!store.detailContent) return null;
  return store.detailContent.split("\n")[0];
});

async function copyToClipboard(text: string, label: string) {
  await navigator.clipboard.writeText(text);
  copied.value = label;
  setTimeout(() => {
    copied.value = "";
  }, 1500);
}

const totpProgress = computed(() => {
  if (!store.totpCode) return 0;
  return (store.totpCode.seconds_remaining / store.totpCode.period) * 100;
});

const tagColors = [
  "var(--accent)",
  "var(--success)",
  "#f9e2af",
  "#fab387",
  "#cba6f7",
  "#f38ba8",
  "#94e2d5",
];

function tagColor(index: number) {
  return tagColors[index % tagColors.length];
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
        <button @click="store.showRaw = !store.showRaw" class="toggle-raw">
          {{ store.showRaw ? "Structured" : "Raw" }}
        </button>
        <button @click="copyToClipboard(firstLine!, 'password')" :disabled="!firstLine">
          Copy Password
        </button>
        <button @click="copyToClipboard(store.detailContent!, 'all')" :disabled="!store.detailContent">
          Copy All
        </button>
        <span v-if="copied" class="copied-msg">Copied!</span>
      </div>
    </div>

    <div v-if="store.detailLoading" class="status">Decrypting...</div>

    <!-- Raw view -->
    <pre v-else-if="store.showRaw" class="detail-content">{{ store.detailContent }}</pre>

    <!-- Structured view -->
    <div v-else-if="store.entryFields" class="fields">
      <!-- Password -->
      <div class="field-row">
        <span class="field-label">Password</span>
        <div class="field-value password-field">
          <code v-if="showPassword">{{ store.entryFields.password }}</code>
          <code v-else class="masked">{{ "\u2022".repeat(Math.min(store.entryFields.password.length, 20)) }}</code>
          <button class="icon-btn" @click="showPassword = !showPassword" :title="showPassword ? 'Hide' : 'Show'">
            {{ showPassword ? "\u{1F441}" : "\u{1F441}\u200D\u{1F5E8}" }}
          </button>
          <button class="icon-btn" @click="copyToClipboard(store.entryFields.password, 'password')" title="Copy">
            \u{1F4CB}
          </button>
        </div>
      </div>

      <!-- Username -->
      <div v-if="store.entryFields.username" class="field-row">
        <span class="field-label">Username</span>
        <div class="field-value">
          <code>{{ store.entryFields.username }}</code>
          <button class="icon-btn" @click="copyToClipboard(store.entryFields.username!, 'username')" title="Copy">
            \u{1F4CB}
          </button>
        </div>
      </div>

      <!-- URL -->
      <div v-if="store.entryFields.url" class="field-row">
        <span class="field-label">URL</span>
        <div class="field-value">
          <a :href="store.entryFields.url" target="_blank" rel="noopener" class="url-link">
            {{ store.entryFields.url }}
          </a>
          <button class="icon-btn" @click="copyToClipboard(store.entryFields.url!, 'url')" title="Copy">
            \u{1F4CB}
          </button>
        </div>
      </div>

      <!-- TOTP -->
      <div v-if="store.totpCode" class="field-row totp-row">
        <span class="field-label">TOTP</span>
        <div class="field-value totp-field">
          <code class="totp-code">{{ store.totpCode.code }}</code>
          <div class="totp-timer">
            <svg viewBox="0 0 36 36" class="totp-ring">
              <path
                class="totp-ring-bg"
                d="M18 2.0845 a 15.9155 15.9155 0 0 1 0 31.831 a 15.9155 15.9155 0 0 1 0 -31.831"
              />
              <path
                class="totp-ring-fg"
                :stroke-dasharray="`${totpProgress}, 100`"
                d="M18 2.0845 a 15.9155 15.9155 0 0 1 0 31.831 a 15.9155 15.9155 0 0 1 0 -31.831"
              />
            </svg>
            <span class="totp-seconds">{{ store.totpCode.seconds_remaining }}s</span>
          </div>
          <button class="icon-btn" @click="copyToClipboard(store.totpCode!.code, 'totp')" title="Copy">
            \u{1F4CB}
          </button>
        </div>
      </div>

      <!-- Tags -->
      <div v-if="store.entryFields.tags.length > 0" class="field-row">
        <span class="field-label">Tags</span>
        <div class="field-value tags-field">
          <span
            v-for="(tag, i) in store.entryFields.tags"
            :key="tag"
            class="tag-badge"
            :style="{ background: tagColor(i), color: '#1e1e2e' }"
          >
            {{ tag }}
          </span>
        </div>
      </div>

      <!-- Extra fields -->
      <div v-for="([key, value]) in store.entryFields.extra" :key="key + value" class="field-row">
        <span class="field-label">{{ key || "Extra" }}</span>
        <div class="field-value">
          <code>{{ value }}</code>
        </div>
      </div>

      <!-- Notes -->
      <div v-if="store.entryFields.notes" class="field-row notes-row">
        <span class="field-label">Notes</span>
        <div class="field-value notes-content">
          {{ store.entryFields.notes }}
        </div>
      </div>
    </div>
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
  align-items: center;
}

.toggle-raw {
  font-size: 11px;
  padding: 3px 8px;
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

.fields {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.field-row {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 12px;
  border-radius: var(--radius);
}

.field-row:hover {
  background: var(--bg-hover);
}

.field-label {
  color: var(--text-dim);
  font-size: 12px;
  min-width: 70px;
  flex-shrink: 0;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  font-weight: 600;
}

.field-value {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
  min-width: 0;
}

.field-value code {
  font-family: "SF Mono", Menlo, Monaco, monospace;
  font-size: 13px;
  word-break: break-all;
}

.masked {
  color: var(--text-dim);
  letter-spacing: 2px;
}

.icon-btn {
  background: none;
  border: none;
  padding: 2px 4px;
  cursor: pointer;
  font-size: 14px;
  opacity: 0.5;
  flex-shrink: 0;
}

.icon-btn:hover {
  opacity: 1;
  background: none;
}

.password-field code {
  font-size: 14px;
}

.url-link {
  color: var(--accent);
  text-decoration: none;
  font-size: 13px;
  word-break: break-all;
}

.url-link:hover {
  text-decoration: underline;
}

.totp-row {
  background: var(--bg-surface);
  border: 1px solid var(--border);
}

.totp-code {
  font-size: 24px;
  font-weight: 700;
  letter-spacing: 4px;
  color: var(--accent);
}

.totp-timer {
  position: relative;
  width: 32px;
  height: 32px;
  flex-shrink: 0;
}

.totp-ring {
  width: 100%;
  height: 100%;
  transform: rotate(-90deg);
}

.totp-ring-bg {
  fill: none;
  stroke: var(--border);
  stroke-width: 3;
}

.totp-ring-fg {
  fill: none;
  stroke: var(--accent);
  stroke-width: 3;
  stroke-linecap: round;
  transition: stroke-dasharray 0.5s ease;
}

.totp-seconds {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 9px;
  font-weight: 600;
  color: var(--text-dim);
}

.tags-field {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

.tag-badge {
  padding: 2px 8px;
  border-radius: 10px;
  font-size: 11px;
  font-weight: 600;
}

.notes-row {
  align-items: flex-start;
}

.notes-content {
  white-space: pre-wrap;
  font-size: 13px;
  line-height: 1.5;
  color: var(--text-dim);
}

.status {
  color: var(--text-dim);
}

.copied-msg {
  color: var(--success);
  font-size: 12px;
}
</style>
