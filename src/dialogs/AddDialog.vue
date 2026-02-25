<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { usePasswordStore } from "../stores/password";
import type { EntryFields } from "../types";

const emit = defineEmits<{ close: [] }>();
const store = usePasswordStore();

const path = ref("");
const password = ref("");
const username = ref("");
const url = ref("");
const notes = ref("");
const totpUri = ref("");
const tags = ref("");
const saving = ref(false);
const error = ref<string | null>(null);
const generatedLength = ref(25);
const genNoSymbols = ref(false);

function generateInline() {
  const charset = genNoSymbols.value
    ? "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
    : "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*()-_=+[]{}|;:,.<>?";
  const arr = new Uint32Array(generatedLength.value);
  crypto.getRandomValues(arr);
  password.value = Array.from(arr, (v) => charset[v % charset.length]).join("");
}

async function save() {
  if (!path.value.trim() || !password.value.trim()) return;
  saving.value = true;
  error.value = null;
  try {
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
      path: path.value.trim(),
      fields,
    });
    await store.loadTree();
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
    <div class="modal add-modal">
      <h3>Add Password</h3>
      <div class="field">
        <label>Path (e.g. folder/name)</label>
        <input v-model="path" placeholder="email/personal" @keyup.enter="save" />
      </div>
      <div class="field">
        <label>Password</label>
        <div class="password-input-row">
          <input v-model="password" type="text" placeholder="Enter or generate" />
          <div class="gen-inline">
            <input
              v-model.number="generatedLength"
              type="number"
              min="8"
              max="128"
              class="gen-length"
              title="Length"
            />
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
        <label>TOTP URI (optional)</label>
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
      <div v-if="error" class="status error" style="color: var(--danger); margin-bottom: 8px;">{{ error }}</div>
      <div class="actions">
        <button @click="emit('close')">Cancel</button>
        <button class="primary" @click="save" :disabled="saving || !path.trim() || !password.trim()">
          {{ saving ? "Saving..." : "Save" }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.add-modal {
  max-width: 520px;
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
</style>
