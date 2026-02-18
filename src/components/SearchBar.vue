<script setup lang="ts">
import { usePasswordStore } from "../stores/password";

const store = usePasswordStore();
</script>

<template>
  <div class="search-bar">
    <input
      v-model="store.searchQuery"
      type="text"
      placeholder="Filter entries..."
    />
  </div>
</template>

<style scoped>
.search-bar {
  padding: 8px;
  border-bottom: 1px solid var(--border);
}

.search-bar input {
  width: 100%;
}
</style>
