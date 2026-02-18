<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

const emit = defineEmits<{ close: [] }>();

const pattern = ref("");
const mode = ref<"grep" | "find">("grep");
const searching = ref(false);
const result = ref<string | null>(null);
const error = ref<string | null>(null);

async function search() {
  if (!pattern.value.trim()) return;
  searching.value = true;
  error.value = null;
  result.value = null;
  try {
    if (mode.value === "grep") {
      result.value = await invoke<string>("grep_passwords", {
        pattern: pattern.value.trim(),
      });
    } else {
      result.value = await invoke<string>("find_passwords", {
        pattern: pattern.value.trim(),
      });
    }
    if (!result.value?.trim()) {
      result.value = "(no results)";
    }
  } catch (e: any) {
    error.value = typeof e === "string" ? e : e.message;
  } finally {
    searching.value = false;
  }
}
</script>

<template>
  <div class="modal-overlay" @click.self="emit('close')">
    <div class="modal" style="max-width: 600px;">
      <h3>Search</h3>
      <div class="checkbox-row">
        <label>
          <input type="radio" value="grep" v-model="mode" /> Grep (search content)
        </label>
        <label style="margin-left: 12px;">
          <input type="radio" value="find" v-model="mode" /> Find (search names)
        </label>
      </div>
      <div class="field">
        <label>Pattern</label>
        <input v-model="pattern" placeholder="search term" @keyup.enter="search" />
      </div>
      <div v-if="result" style="margin-bottom: 8px;">
        <pre style="background: var(--bg); padding: 8px; border-radius: var(--radius); font-size: 12px; white-space: pre-wrap; max-height: 300px; overflow-y: auto;">{{ result }}</pre>
      </div>
      <div v-if="error" style="color: var(--danger); margin-bottom: 8px;">{{ error }}</div>
      <div class="actions">
        <button @click="emit('close')">Close</button>
        <button class="primary" @click="search" :disabled="searching || !pattern.trim()">
          {{ searching ? "Searching..." : "Search" }}
        </button>
      </div>
    </div>
  </div>
</template>
