<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { usePasswordStore } from "../stores/password";

const emit = defineEmits<{ close: [] }>();
const store = usePasswordStore();

const filePath = ref("");
const format = ref("Chrome");
const importing = ref(false);
const result = ref<string | null>(null);
const error = ref<string | null>(null);

const formats = ["Chrome", "Firefox", "OnePassword", "Bitwarden", "KeePass", "LastPass"];

async function selectFile() {
  const selected = await open({
    multiple: false,
    filters: [{ name: "CSV", extensions: ["csv"] }],
  });
  if (selected) {
    filePath.value = selected as string;
  }
}

async function doImport() {
  if (!filePath.value) return;
  importing.value = true;
  error.value = null;
  result.value = null;
  try {
    const count = await invoke<number>("import_csv", {
      filePath: filePath.value,
      format: format.value,
    });
    result.value = `Successfully imported ${count} entries.`;
    await store.loadTree();
  } catch (e: any) {
    error.value = typeof e === "string" ? e : e.message;
  } finally {
    importing.value = false;
  }
}
</script>

<template>
  <div class="modal-overlay" @click.self="emit('close')">
    <div class="modal">
      <h3>Import Passwords</h3>
      <div class="field">
        <label>Source Format</label>
        <select v-model="format">
          <option v-for="f in formats" :key="f" :value="f">{{ f }}</option>
        </select>
      </div>
      <div class="field">
        <label>CSV File</label>
        <div class="file-row">
          <input v-model="filePath" placeholder="Select a CSV file..." readonly />
          <button @click="selectFile">Browse</button>
        </div>
      </div>
      <div v-if="result" class="success">{{ result }}</div>
      <div v-if="error" style="color: var(--danger); margin-bottom: 8px;">{{ error }}</div>
      <div class="actions">
        <button @click="emit('close')">{{ result ? "Done" : "Cancel" }}</button>
        <button class="primary" @click="doImport" :disabled="importing || !filePath">
          {{ importing ? "Importing..." : "Import" }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.file-row {
  display: flex;
  gap: 6px;
}

.file-row input {
  flex: 1;
}

.success {
  color: var(--success);
  margin-bottom: 8px;
  font-size: 13px;
}
</style>
