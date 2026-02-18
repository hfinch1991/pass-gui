<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { usePasswordStore } from "../stores/password";

const emit = defineEmits<{ close: [] }>();
const store = usePasswordStore();

const path = ref("");
const content = ref("");
const saving = ref(false);
const error = ref<string | null>(null);

async function save() {
  if (!path.value.trim() || !content.value.trim()) return;
  saving.value = true;
  error.value = null;
  try {
    await invoke("insert_password", {
      path: path.value.trim(),
      content: content.value,
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
    <div class="modal">
      <h3>Add Password</h3>
      <div class="field">
        <label>Path (e.g. folder/name)</label>
        <input v-model="path" placeholder="email/personal" @keyup.enter="save" />
      </div>
      <div class="field">
        <label>Content (first line = password)</label>
        <textarea v-model="content" rows="5" placeholder="password&#10;username: user@example.com&#10;url: https://example.com"></textarea>
      </div>
      <div v-if="error" class="status error" style="color: var(--danger); margin-bottom: 8px;">{{ error }}</div>
      <div class="actions">
        <button @click="emit('close')">Cancel</button>
        <button class="primary" @click="save" :disabled="saving || !path.trim() || !content.trim()">
          {{ saving ? "Saving..." : "Save" }}
        </button>
      </div>
    </div>
  </div>
</template>
