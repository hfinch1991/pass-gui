<script setup lang="ts">
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { usePasswordStore } from "../stores/password";
import AddDialog from "../dialogs/AddDialog.vue";
import EditDialog from "../dialogs/EditDialog.vue";
import GenerateDialog from "../dialogs/GenerateDialog.vue";
import MoveDialog from "../dialogs/MoveDialog.vue";
import GrepDialog from "../dialogs/GrepDialog.vue";
import SettingsDialog from "../dialogs/SettingsDialog.vue";

const store = usePasswordStore();
const showAdd = ref(false);
const showEdit = ref(false);
const showGenerate = ref(false);
const showMove = ref(false);
const showGrep = ref(false);
const showSettings = ref(false);
const showConfirmDelete = ref(false);
const showMessage = ref(false);
const messageText = ref("");
const deleteError = ref<string | null>(null);
const deleting = ref(false);

const selectedCount = computed(() => store.checkedPaths.size);
const canEdit = computed(() => store.selectedPath !== null);
const deletePaths = computed(() => Array.from(store.checkedPaths));

function requestDelete() {
  if (selectedCount.value === 0) return;
  deleteError.value = null;
  showConfirmDelete.value = true;
}

async function confirmDelete() {
  deleting.value = true;
  deleteError.value = null;
  try {
    await store.deleteSelected();
    showConfirmDelete.value = false;
  } catch (e: any) {
    deleteError.value = typeof e === "string" ? e : e.message;
  } finally {
    deleting.value = false;
  }
}

function showMsg(text: string) {
  messageText.value = text;
  showMessage.value = true;
}

async function gitPush() {
  try {
    const out = await invoke<string>("git_push");
    showMsg("Git push completed:\n" + out);
  } catch (e: any) {
    showMsg("Git push failed: " + (typeof e === "string" ? e : e.message));
  }
}

async function gitPull() {
  try {
    const out = await invoke<string>("git_pull");
    await store.loadTree();
    showMsg("Git pull completed:\n" + out);
  } catch (e: any) {
    showMsg("Git pull failed: " + (typeof e === "string" ? e : e.message));
  }
}

async function gitLog() {
  try {
    const log = await invoke<string>("git_log");
    showMsg("Recent commits:\n\n" + log);
  } catch (e: any) {
    showMsg("Git log failed: " + (typeof e === "string" ? e : e.message));
  }
}
</script>

<template>
  <div class="toolbar">
    <div class="toolbar-group">
      <button class="primary" @click="showAdd = true">+ Add</button>
      <button @click="showGenerate = true">Generate</button>
      <button @click="showEdit = true" :disabled="!canEdit">Edit</button>
      <button @click="showMove = true" :disabled="!canEdit">Move</button>
      <button
        class="danger"
        @click="requestDelete"
        :disabled="selectedCount === 0"
      >
        Delete ({{ selectedCount }})
      </button>
    </div>
    <div class="toolbar-group">
      <button @click="showGrep = true">Grep</button>
      <button @click="store.loadTree()">Refresh</button>
      <button @click="gitPush">Push</button>
      <button @click="gitPull">Pull</button>
      <button @click="gitLog">Log</button>
      <button @click="showSettings = true">Settings</button>
    </div>
  </div>

  <AddDialog v-if="showAdd" @close="showAdd = false" />
  <EditDialog v-if="showEdit" @close="showEdit = false" />
  <GenerateDialog v-if="showGenerate" @close="showGenerate = false" />
  <MoveDialog v-if="showMove" @close="showMove = false" />
  <GrepDialog v-if="showGrep" @close="showGrep = false" />
  <SettingsDialog v-if="showSettings" @close="showSettings = false" />

  <!-- Confirm Delete Dialog -->
  <div v-if="showConfirmDelete" class="modal-overlay" @click.self="showConfirmDelete = false">
    <div class="modal">
      <h3>Confirm Delete</h3>
      <p style="margin-bottom: 8px;">Delete {{ deletePaths.length }} selected entries?</p>
      <ul style="margin-bottom: 12px; padding-left: 20px; font-size: 12px; color: var(--text-dim); max-height: 200px; overflow-y: auto;">
        <li v-for="p in deletePaths" :key="p">{{ p }}</li>
      </ul>
      <div v-if="deleteError" style="color: var(--danger); margin-bottom: 8px;">{{ deleteError }}</div>
      <div class="actions">
        <button @click="showConfirmDelete = false">Cancel</button>
        <button class="danger" @click="confirmDelete" :disabled="deleting">
          {{ deleting ? "Deleting..." : "Delete" }}
        </button>
      </div>
    </div>
  </div>

  <!-- Message Dialog -->
  <div v-if="showMessage" class="modal-overlay" @click.self="showMessage = false">
    <div class="modal">
      <pre style="white-space: pre-wrap; font-size: 12px; margin-bottom: 12px;">{{ messageText }}</pre>
      <div class="actions">
        <button class="primary" @click="showMessage = false">OK</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  gap: 8px;
  flex-wrap: wrap;
}

.toolbar-group {
  display: flex;
  gap: 6px;
  align-items: center;
}
</style>
