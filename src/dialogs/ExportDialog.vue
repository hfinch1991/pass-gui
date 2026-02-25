<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";

const emit = defineEmits<{ close: [] }>();

const exporting = ref(false);
const result = ref<string | null>(null);
const error = ref<string | null>(null);

async function doExport() {
  const filePath = await save({
    filters: [{ name: "CSV", extensions: ["csv"] }],
    defaultPath: "passwords.csv",
  });
  if (!filePath) return;

  exporting.value = true;
  error.value = null;
  result.value = null;
  try {
    const count = await invoke<number>("export_csv", { filePath });
    result.value = `Successfully exported ${count} entries.`;
  } catch (e: any) {
    error.value = typeof e === "string" ? e : e.message;
  } finally {
    exporting.value = false;
  }
}
</script>

<template>
  <div class="modal-overlay" @click.self="emit('close')">
    <div class="modal">
      <h3>Export Passwords</h3>
      <div class="warning">
        The exported CSV will contain all your passwords in plaintext.
        Store it securely and delete it when no longer needed.
      </div>
      <div v-if="result" class="success">{{ result }}</div>
      <div v-if="error" style="color: var(--danger); margin-bottom: 8px;">{{ error }}</div>
      <div class="actions">
        <button @click="emit('close')">{{ result ? "Done" : "Cancel" }}</button>
        <button class="danger" @click="doExport" :disabled="exporting">
          {{ exporting ? "Exporting..." : "Export to CSV" }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.warning {
  background: rgba(243, 139, 168, 0.15);
  border: 1px solid var(--danger);
  border-radius: var(--radius);
  padding: 10px 12px;
  font-size: 12px;
  line-height: 1.5;
  margin-bottom: 12px;
  color: var(--danger);
}

.success {
  color: var(--success);
  margin-bottom: 8px;
  font-size: 13px;
}
</style>
