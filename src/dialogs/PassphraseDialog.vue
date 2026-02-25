<script setup lang="ts">
import { ref, onMounted } from "vue";

const props = defineProps<{
  error?: string | null;
}>();

const emit = defineEmits<{
  (e: "submit", passphrase: string): void;
  (e: "cancel"): void;
}>();

const passphrase = ref("");
const inputRef = ref<HTMLInputElement | null>(null);

function submit() {
  if (passphrase.value) {
    emit("submit", passphrase.value);
  }
}

onMounted(() => {
  inputRef.value?.focus();
});
</script>

<template>
  <div class="modal-overlay" @click.self="emit('cancel')">
    <div class="modal">
      <h3>GPG Passphrase Required</h3>
      <p style="margin-bottom: 12px; font-size: 13px;">Please enter the passphrase to unlock your GPG key.</p>
      
      <div v-if="error" style="color: var(--danger); margin-bottom: 12px; font-size: 12px;">
        {{ error }}
      </div>

      <div class="field">
        <input
          ref="inputRef"
          v-model="passphrase"
          type="password"
          placeholder="Passphrase"
          @keydown.enter="submit"
        />
      </div>

      <div class="actions">
        <button @click="emit('cancel')">Cancel</button>
        <button class="primary" @click="submit" :disabled="!passphrase">Unlock</button>
      </div>
    </div>
  </div>
</template>
