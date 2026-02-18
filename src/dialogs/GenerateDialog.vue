<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { usePasswordStore } from "../stores/password";

const emit = defineEmits<{ close: [] }>();
const store = usePasswordStore();

const path = ref("");
const length = ref(25);
const noSymbols = ref(false);
const generating = ref(false);
const result = ref<string | null>(null);
const error = ref<string | null>(null);

async function generate() {
  if (!path.value.trim()) return;
  generating.value = true;
  error.value = null;
  result.value = null;
  try {
    result.value = await invoke<string>("generate_password", {
      path: path.value.trim(),
      length: length.value,
      noSymbols: noSymbols.value,
    });
    await store.loadTree();
  } catch (e: any) {
    error.value = typeof e === "string" ? e : e.message;
  } finally {
    generating.value = false;
  }
}
</script>

<template>
  <div class="modal-overlay" @click.self="emit('close')">
    <div class="modal">
      <h3>Generate Password</h3>
      <div class="field">
        <label>Path</label>
        <input v-model="path" placeholder="folder/name" />
      </div>
      <div class="field">
        <label>Length</label>
        <input v-model.number="length" type="number" min="1" max="128" />
      </div>
      <div class="checkbox-row">
        <input type="checkbox" id="no-symbols" v-model="noSymbols" />
        <label for="no-symbols">No symbols (alphanumeric only)</label>
      </div>
      <div v-if="result" style="margin-bottom: 8px;">
        <pre style="background: var(--bg); padding: 8px; border-radius: var(--radius); font-size: 12px; white-space: pre-wrap;">{{ result }}</pre>
      </div>
      <div v-if="error" style="color: var(--danger); margin-bottom: 8px;">{{ error }}</div>
      <div class="actions">
        <button @click="emit('close')">{{ result ? 'Done' : 'Cancel' }}</button>
        <button class="primary" @click="generate" :disabled="generating || !path.trim()">
          {{ generating ? "Generating..." : "Generate" }}
        </button>
      </div>
    </div>
  </div>
</template>
