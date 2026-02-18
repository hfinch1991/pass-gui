<script setup lang="ts">
import { onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { SetupStatus, StoreInfo, GpgKey } from "../types";

const emit = defineEmits<{ close: [] }>();

const status = ref<SetupStatus | null>(null);
const storeInfo = ref<StoreInfo | null>(null);
const gpgKeys = ref<GpgKey[]>([]);
const loading = ref(true);

const editingRemote = ref(false);
const remoteUrl = ref("");
const remoteSaving = ref(false);
const remoteError = ref<string | null>(null);
const remoteSuccess = ref(false);

onMounted(async () => {
  try {
    const [s, info, keys] = await Promise.all([
      invoke<SetupStatus>("check_setup_status"),
      invoke<StoreInfo>("get_store_info"),
      invoke<GpgKey[]>("list_gpg_keys"),
    ]);
    status.value = s;
    storeInfo.value = info;
    gpgKeys.value = keys;
    remoteUrl.value = info.git_remote || "";
  } catch {
    // best effort
  } finally {
    loading.value = false;
  }
});

async function saveRemote() {
  remoteSaving.value = true;
  remoteError.value = null;
  remoteSuccess.value = false;
  try {
    if (storeInfo.value && !storeInfo.value.has_git) {
      await invoke<string>("init_git_repo");
    }
    if (remoteUrl.value.trim()) {
      await invoke<string>("add_git_remote", { url: remoteUrl.value.trim() });
    }
    storeInfo.value = await invoke<StoreInfo>("get_store_info");
    remoteSuccess.value = true;
    editingRemote.value = false;
  } catch (e: any) {
    remoteError.value = typeof e === "string" ? e : e.message || "Failed";
  } finally {
    remoteSaving.value = false;
  }
}

function currentGpgKey(): GpgKey | undefined {
  if (!storeInfo.value?.gpg_id) return undefined;
  return gpgKeys.value.find(
    (k) => k.key_id === storeInfo.value!.gpg_id || k.fingerprint.endsWith(storeInfo.value!.gpg_id!)
  );
}
</script>

<template>
  <div class="modal-overlay" @click.self="emit('close')">
    <div class="modal settings-modal">
      <h3>Settings</h3>

      <div v-if="loading" class="status">Loading...</div>

      <template v-else>
        <!-- Dependencies -->
        <section class="section">
          <h4>Dependencies</h4>
          <div class="dep-list">
            <div
              v-for="dep in status?.dependencies"
              :key="dep.name"
              class="dep-row"
            >
              <span :class="dep.installed ? 'ok' : 'missing'">
                {{ dep.installed ? "\u2713" : "\u2717" }}
              </span>
              <span class="dep-name">{{ dep.name }}</span>
              <span class="dep-version">{{ dep.version || "not found" }}</span>
            </div>
          </div>
        </section>

        <!-- Store Info -->
        <section class="section">
          <h4>Password Store</h4>
          <div class="info-grid">
            <div class="info-row">
              <span class="label">Path</span>
              <code>{{ storeInfo?.store_path || "~/.password-store" }}</code>
            </div>
            <div class="info-row">
              <span class="label">GPG Key</span>
              <span v-if="currentGpgKey()">
                {{ currentGpgKey()!.uid }}
                <span class="dim">({{ storeInfo?.gpg_id }})</span>
              </span>
              <span v-else-if="storeInfo?.gpg_id">
                {{ storeInfo.gpg_id }}
              </span>
              <span v-else class="dim">Not configured</span>
            </div>
          </div>
        </section>

        <!-- Git -->
        <section class="section">
          <h4>Git Sync</h4>
          <div class="info-grid">
            <div class="info-row">
              <span class="label">Status</span>
              <span>{{ storeInfo?.has_git ? "Enabled" : "Not initialized" }}</span>
            </div>
            <div class="info-row">
              <span class="label">Remote</span>
              <template v-if="!editingRemote">
                <code v-if="storeInfo?.git_remote">{{ storeInfo.git_remote }}</code>
                <span v-else class="dim">None</span>
                <button class="small" @click="editingRemote = true">Edit</button>
              </template>
              <template v-else>
                <div class="remote-edit">
                  <input
                    v-model="remoteUrl"
                    type="text"
                    placeholder="git@github.com:user/pass-store.git"
                  />
                  <div class="remote-actions">
                    <button class="primary small" :disabled="remoteSaving" @click="saveRemote">
                      {{ remoteSaving ? "Saving..." : "Save" }}
                    </button>
                    <button class="small" @click="editingRemote = false; remoteError = null">
                      Cancel
                    </button>
                  </div>
                  <div v-if="remoteError" class="error">{{ remoteError }}</div>
                </div>
              </template>
            </div>
            <div v-if="remoteSuccess" class="success-msg">Remote updated.</div>
          </div>
        </section>
      </template>

      <div class="actions">
        <button class="primary" @click="emit('close')">Close</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.settings-modal {
  min-width: 460px;
  max-width: 540px;
}
.section {
  margin-bottom: 16px;
}
.section h4 {
  font-size: 13px;
  color: var(--text-dim);
  margin-bottom: 8px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  font-weight: 600;
}
.dep-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.dep-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 8px;
  font-size: 13px;
}
.dep-name {
  font-weight: 600;
  min-width: 50px;
}
.dep-version {
  color: var(--text-dim);
  font-size: 12px;
}
.ok {
  color: var(--success);
  font-weight: bold;
}
.missing {
  color: var(--danger);
  font-weight: bold;
}
.info-grid {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.info-row {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
}
.info-row .label {
  color: var(--text-dim);
  min-width: 60px;
  flex-shrink: 0;
}
code {
  background: var(--bg);
  padding: 2px 6px;
  border-radius: 3px;
  font-size: 12px;
  word-break: break-all;
}
.dim {
  color: var(--text-dim);
  font-size: 12px;
}
.small {
  padding: 2px 8px;
  font-size: 11px;
}
.remote-edit {
  display: flex;
  flex-direction: column;
  gap: 6px;
  flex: 1;
}
.remote-edit input {
  width: 100%;
}
.remote-actions {
  display: flex;
  gap: 6px;
}
.error {
  color: var(--danger);
  font-size: 12px;
}
.success-msg {
  color: var(--success);
  font-size: 12px;
  padding-left: 68px;
}
.status {
  color: var(--text-dim);
  padding: 12px 0;
}
</style>
