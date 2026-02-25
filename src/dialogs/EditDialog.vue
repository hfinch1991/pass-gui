<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { usePasswordStore } from "../stores/password";
import type { EntryFields } from "../types";

const emit = defineEmits<{ close: [] }>();
const store = usePasswordStore();

const password = ref("");
const username = ref("");
const url = ref("");
const notes = ref("");
const totpUri = ref("");
const tags = ref("");
const rawContent = ref("");
const editRaw = ref(false);
const saving = ref(false);
const loading = ref(true);
const error = ref<string | null>(null);
const generatedLength = ref(25);
const genNoSymbols = ref(false);

onMounted(async () => {
  if (!store.selectedPath) return;
  try {
    const pw = store.cachedPassphrase;
    const [fields, raw] = await Promise.all([
      invoke<EntryFields>("get_entry_fields", { path: store.selectedPath, passphrase: pw }),
      invoke<string>("show_password", { path: store.selectedPath, passphrase: pw }),
    ]);
    password.value = fields.password;
    username.value = fields.username || "";
    url.value = fields.url || "";
    notes.value = fields.notes || "";
    totpUri.value = fields.totp || "";
    tags.value = fields.tags.join(", ");
    rawContent.value = raw;
  } catch (e: any) {
    const err = typeof e === "string" ? e : e.message || "Decryption failed";
    if (err.includes("GPG decryption failed") || err.includes("bad passphrase") || err.includes("decryption failed")) {
      store.needsPassphrase = true;
      store.pendingAction = async () => {
        // Re-run onMounted logic
        loading.value = true;
        error.value = null;
        try {
          const pw = store.cachedPassphrase;
          const [fields, raw] = await Promise.all([
            invoke<EntryFields>("get_entry_fields", { path: store.selectedPath!, passphrase: pw }),
            invoke<string>("show_password", { path: store.selectedPath!, passphrase: pw }),
          ]);
          password.value = fields.password;
          username.value = fields.username || "";
          url.value = fields.url || "";
          notes.value = fields.notes || "";
          totpUri.value = fields.totp || "";
          tags.value = fields.tags.join(", ");
          rawContent.value = raw;
        } catch (e2: any) {
          error.value = typeof e2 === "string" ? e2 : e2.message;
        } finally {
          loading.value = false;
        }
      };
    } else {
      error.value = err;
    }
  } finally {
    loading.value = false;
  }
});

function generateInline() {
  const charset = genNoSymbols.value
    ? "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
    : "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*()-_=+[]{}|;:,.<>?";
  const arr = new Uint32Array(generatedLength.value);
  crypto.getRandomValues(arr);
  password.value = Array.from(arr, (v) => charset[v % charset.length]).join("");
}

async function save() {
  if (!store.selectedPath) return;
  saving.value = true;
  error.value = null;
  try {
    if (editRaw.value) {
      await invoke("insert_password", {
        path: store.selectedPath,
        content: rawContent.value,
      });
    } else {
      const fields: EntryFields = {
        password: password.value,
        username: username.value.trim() || null,
        url: url.value.trim() || null,
        notes: notes.value.trim() || null,
        totp: totpUri.value.trim() || null,
        tags: tags.value
          .split(",")
          .map((t) => t.trim())
          .filter((t) => t.length > 0),
        extra: [],
      };
      await invoke("save_entry_fields", {
        path: store.selectedPath,
        fields,
      });
    }
    await store.showPassword(store.selectedPath);
    emit("close");
  } catch (e: any) {
    error.value = typeof e === "string" ? e : e.message;
  } finally {
    saving.value = false;
  }
}
</script>

<template>
  <div class="modal-overlay" @click.self="emit('close')">
    <div class="modal edit-modal">
      <div class="edit-header">
        <h3>Edit: {{ store.selectedPath }}</h3>
        <button class="toggle-raw" @click="editRaw = !editRaw">
          {{ editRaw ? "Structured" : "Raw" }}
        </button>
      </div>

      <div v-if="loading" class="status">Loading...</div>
      <template v-else>
        <!-- Raw editing mode -->
        <div v-if="editRaw" class="field">
          <label>Content</label>
          <textarea v-model="rawContent" rows="10"></textarea>
        </div>

        <!-- Structured editing mode -->
        <template v-else>
          <div class="field">
            <label>Password</label>
            <div class="password-input-row">
              <input v-model="password" type="text" />
              <div class="gen-inline">
                <input v-model.number="generatedLength" type="number" min="8" max="128" class="gen-length" title="Length" />
                <label class="gen-checkbox">
                  <input type="checkbox" v-model="genNoSymbols" />
                  A-z
                </label>
                <button type="button" @click="generateInline" class="gen-btn">Gen</button>
              </div>
            </div>
          </div>
          <div class="field">
            <label>Username</label>
            <input v-model="username" placeholder="user@example.com" />
          </div>
          <div class="field">
            <label>URL</label>
            <input v-model="url" placeholder="https://example.com" />
          </div>
          <div class="field">
            <label>TOTP URI</label>
            <input v-model="totpUri" placeholder="otpauth://totp/..." />
          </div>
          <div class="field">
            <label>Tags (comma-separated)</label>
            <input v-model="tags" placeholder="work, email" />
          </div>
          <div class="field">
            <label>Notes</label>
            <textarea v-model="notes" rows="3" placeholder="Additional notes..."></textarea>
          </div>
        </template>

        <div v-if="error" style="color: var(--danger); margin-bottom: 8px;">{{ error }}</div>
        <div class="actions">
          <button @click="emit('close')">Cancel</button>
          <button class="primary" @click="save" :disabled="saving">
            {{ saving ? "Saving..." : "Save" }}
          </button>
        </div>
      </template>
    </div>
  </div>
</template>

<style scoped>
.edit-modal {
  max-width: 520px;
}

.edit-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}

.edit-header h3 {
  font-size: 15px;
}

.toggle-raw {
  font-size: 11px;
  padding: 3px 8px;
}

.password-input-row {
  display: flex;
  gap: 6px;
  align-items: center;
}

.password-input-row > input {
  flex: 1;
}

.gen-inline {
  display: flex;
  align-items: center;
  gap: 4px;
  flex-shrink: 0;
}

.gen-length {
  width: 48px;
  padding: 4px;
  font-size: 12px;
  text-align: center;
}

.gen-checkbox {
  font-size: 11px;
  display: flex;
  align-items: center;
  gap: 2px;
  color: var(--text-dim);
  cursor: pointer;
  white-space: nowrap;
}

.gen-checkbox input {
  width: auto;
}

.gen-btn {
  padding: 4px 8px;
  font-size: 11px;
}

.status {
  color: var(--text-dim);
  padding: 12px 0;
}
</style>
