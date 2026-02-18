<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useSetupStore } from "../stores/setup";

const setup = useSetupStore();

const newName = ref("");
const newEmail = ref("");
const newPassphrase = ref("");

onMounted(() => {
  setup.loadGpgKeys();
});

async function handleGenerate() {
  if (!newName.value.trim() || !newEmail.value.trim()) return;
  await setup.generateKey(newName.value.trim(), newEmail.value.trim(), newPassphrase.value);
}
</script>

<template>
  <div class="step">
    <h3>GPG Key</h3>
    <p class="step-desc">
      Select an existing GPG key or generate a new one for encrypting your
      passwords.
    </p>

    <div v-if="setup.keysLoading" class="status">Loading GPG keys...</div>

    <template v-else>
      <div v-if="setup.gpgKeys.length > 0" class="field">
        <label>Select a key</label>
        <select v-model="setup.selectedKeyId">
          <option :value="null" disabled>-- Choose a key --</option>
          <option v-for="key in setup.gpgKeys" :key="key.key_id" :value="key.key_id">
            {{ key.uid }} ({{ key.key_id.slice(-8) }})
          </option>
        </select>
      </div>

      <div v-else class="no-keys">
        <p>No GPG keys found. Generate one below.</p>
      </div>

      <details class="generate-section" :open="setup.gpgKeys.length === 0 ? true : undefined">
        <summary>Generate a new GPG key</summary>
        <div class="gen-form">
          <div class="field">
            <label>Full Name</label>
            <input v-model="newName" type="text" placeholder="John Doe" />
          </div>
          <div class="field">
            <label>Email</label>
            <input v-model="newEmail" type="email" placeholder="john@example.com" />
          </div>
          <div class="field">
            <label>Passphrase (optional)</label>
            <input v-model="newPassphrase" type="password" placeholder="Leave empty for no passphrase" />
          </div>
          <button
            class="primary"
            :disabled="setup.genKeyLoading || !newName.trim() || !newEmail.trim()"
            @click="handleGenerate"
          >
            {{ setup.genKeyLoading ? "Generating... (this may take a moment)" : "Generate GPG Key" }}
          </button>
          <div v-if="setup.genKeyError" class="error">{{ setup.genKeyError }}</div>
        </div>
      </details>
    </template>

    <div v-if="setup.keysError" class="error">{{ setup.keysError }}</div>
  </div>
</template>

<style scoped>
.step {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.step-desc {
  color: var(--text-dim);
  font-size: 13px;
  line-height: 1.5;
}
.field {
  margin-bottom: 10px;
}
.field label {
  display: block;
  margin-bottom: 4px;
  font-size: 12px;
  color: var(--text-dim);
}
.field select,
.field input {
  width: 100%;
}
.no-keys p {
  color: var(--text-dim);
  font-size: 13px;
}
.generate-section {
  background: var(--bg);
  border-radius: var(--radius);
  padding: 12px;
}
.generate-section summary {
  cursor: pointer;
  font-size: 13px;
  color: var(--accent);
}
.gen-form {
  margin-top: 12px;
}
.error {
  color: var(--danger);
  font-size: 12px;
  margin-top: 4px;
}
.status {
  color: var(--text-dim);
}
</style>
