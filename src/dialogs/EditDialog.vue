<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { usePasswordStore } from "../stores/password";

const emit = defineEmits<{ close: [] }>();
const store = usePasswordStore();

const content = ref("");
const saving = ref(false);
const loading = ref(true);
const error = ref<string | null>(null);

onMounted(async () => {
  if (!store.selectedPath) return;
  try {
    content.value = await invoke<string>("show_password", {
      path: store.selectedPath,
    });
  } catch (e: any) {
    error.value = typeof e === "string" ? e : e.message;
  } finally {
    loading.value = false;
  }
});

async function save() {
  if (!store.selectedPath || !content.value) return;
  saving.value = true;
  error.value = null;
  try {
    await invoke("insert_password", {
      path: store.selectedPath,
      content: content.value,
    });
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
    <div class="modal">
      <h3>Edit: {{ store.selectedPath }}</h3>
      <div v-if="loading" class="status">Loading...</div>
      <template v-else>
        <div class="field">
          <label>Content</label>
          <textarea v-model="content" rows="8"></textarea>
        </div>
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
