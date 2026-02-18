<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { usePasswordStore } from "../stores/password";

const emit = defineEmits<{ close: [] }>();
const store = usePasswordStore();

const newPath = ref("");
const mode = ref<"move" | "copy">("move");
const saving = ref(false);
const error = ref<string | null>(null);

async function execute() {
  if (!store.selectedPath || !newPath.value.trim()) return;
  saving.value = true;
  error.value = null;
  try {
    if (mode.value === "move") {
      await invoke("move_password", {
        oldPath: store.selectedPath,
        newPath: newPath.value.trim(),
      });
    } else {
      await invoke("copy_password", {
        oldPath: store.selectedPath,
        newPath: newPath.value.trim(),
      });
    }
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
    <div class="modal">
      <h3>Move / Copy</h3>
      <div class="field">
        <label>From</label>
        <input :value="store.selectedPath" disabled />
      </div>
      <div class="field">
        <label>To</label>
        <input v-model="newPath" placeholder="new/path" @keyup.enter="execute" />
      </div>
      <div class="checkbox-row">
        <label>
          <input type="radio" value="move" v-model="mode" /> Move
        </label>
        <label style="margin-left: 12px;">
          <input type="radio" value="copy" v-model="mode" /> Copy
        </label>
      </div>
      <div v-if="error" style="color: var(--danger); margin-bottom: 8px;">{{ error }}</div>
      <div class="actions">
        <button @click="emit('close')">Cancel</button>
        <button class="primary" @click="execute" :disabled="saving || !newPath.trim()">
          {{ saving ? "Working..." : mode === "move" ? "Move" : "Copy" }}
        </button>
      </div>
    </div>
  </div>
</template>
