<script setup lang="ts">
import { ref } from "vue";
import { usePasswordStore } from "../stores/password";
import type { TreeNode } from "../types";

const props = defineProps<{
  nodes: TreeNode[];
  depth: number;
}>();

const store = usePasswordStore();
const collapsed = ref<Record<string, boolean>>({});

function toggle(path: string) {
  collapsed.value[path] = !collapsed.value[path];
}

function isCollapsed(path: string) {
  return collapsed.value[path] ?? false;
}

function onEntryClick(node: TreeNode) {
  if (node.is_dir) {
    toggle(node.path);
  } else {
    store.showPassword(node.path);
  }
}

function onCheckboxChange(node: TreeNode, event: Event) {
  event.stopPropagation();
  if (node.is_dir) {
    const allChecked = allChildrenChecked(node);
    store.toggleAllInNode(node, !allChecked);
  } else {
    store.toggleChecked(node.path);
  }
}

function allChildrenChecked(node: TreeNode): boolean {
  if (!node.is_dir) return store.checkedPaths.has(node.path);
  return node.children.length > 0 && node.children.every(allChildrenChecked);
}

function someChildrenChecked(node: TreeNode): boolean {
  if (!node.is_dir) return store.checkedPaths.has(node.path);
  return node.children.some(someChildrenChecked);
}
</script>

<template>
  <ul class="tree-list" :class="{ 'tree-root': depth === 0 }">
    <li v-for="(node, index) in nodes" :key="node.path" class="tree-item">
      <div
        class="tree-row"
        :class="{
          active: !node.is_dir && store.selectedPath === node.path,
          dir: node.is_dir,
        }"
        @click="onEntryClick(node)"
      >
        <!-- Indent guides -->
        <span
          v-for="d in depth"
          :key="d"
          class="indent-guide"
        ></span>

        <!-- Checkbox -->
        <input
          type="checkbox"
          class="tree-checkbox"
          :checked="node.is_dir ? allChildrenChecked(node) : store.checkedPaths.has(node.path)"
          :indeterminate="node.is_dir && someChildrenChecked(node) && !allChildrenChecked(node)"
          @click.stop
          @change="onCheckboxChange(node, $event)"
        />

        <!-- Expand/collapse arrow for folders -->
        <span v-if="node.is_dir" class="tree-arrow" :class="{ collapsed: isCollapsed(node.path) }">
          <svg width="10" height="10" viewBox="0 0 10 10">
            <path d="M3 2 L7 5 L3 8" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
        </span>
        <span v-else class="tree-arrow-spacer"></span>

        <!-- Icon -->
        <span class="tree-icon" :class="{ 'icon-dir': node.is_dir, 'icon-open': node.is_dir && !isCollapsed(node.path) }">
          <svg v-if="node.is_dir && isCollapsed(node.path)" width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
            <path d="M1.5 2A1.5 1.5 0 000 3.5v9A1.5 1.5 0 001.5 14h13a1.5 1.5 0 001.5-1.5V5a1.5 1.5 0 00-1.5-1.5H7.71L6.85 2.15A.5.5 0 006.5 2H1.5z"/>
          </svg>
          <svg v-else-if="node.is_dir" width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
            <path d="M.54 3.87L.5 3a2 2 0 012-2h3.672a2 2 0 011.414.586l.828.828A2 2 0 009.828 3H14a2 2 0 012 2v.5H.54z"/>
            <path d="M14.412 4H1.588l-.757 7.568A2 2 0 002.819 14h10.362a2 2 0 001.988-2.432L14.412 4z"/>
          </svg>
          <svg v-else width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
            <path d="M8 1a2 2 0 012 2v4H6V3a2 2 0 012-2zm3 6V3a3 3 0 00-6 0v4a2 2 0 00-2 2v5a2 2 0 002 2h6a2 2 0 002-2V9a2 2 0 00-2-2z"/>
          </svg>
        </span>

        <!-- Name -->
        <span class="tree-name">{{ node.name }}</span>

        <!-- Entry count badge for folders -->
        <span v-if="node.is_dir" class="tree-badge">{{ node.children.length }}</span>
      </div>

      <transition name="expand">
        <TreeView
          v-if="node.is_dir && !isCollapsed(node.path)"
          :nodes="node.children"
          :depth="depth + 1"
        />
      </transition>
    </li>
  </ul>
</template>

<style scoped>
.tree-list {
  list-style: none;
  margin: 0;
  padding: 0;
}

.tree-root {
  padding: 4px 0;
}

.tree-row {
  display: flex;
  align-items: center;
  gap: 0;
  padding: 4px 8px 4px 4px;
  cursor: pointer;
  user-select: none;
  border-radius: 4px;
  margin: 1px 6px;
  transition: background 0.1s;
}

.tree-row:hover {
  background: var(--bg-hover);
}

.tree-row.active {
  background: rgba(137, 180, 250, 0.15);
  color: var(--accent);
}

.tree-row.active .tree-icon {
  color: var(--accent);
}

/* Indent guides */
.indent-guide {
  display: inline-block;
  width: 20px;
  height: 28px;
  flex-shrink: 0;
  position: relative;
}

.indent-guide::before {
  content: "";
  position: absolute;
  left: 9px;
  top: 0;
  bottom: 0;
  width: 1px;
  background: var(--border);
}

/* Checkbox */
.tree-checkbox {
  width: 13px;
  height: 13px;
  cursor: pointer;
  flex-shrink: 0;
  margin: 0 4px 0 2px;
  accent-color: var(--accent);
}

/* Arrow */
.tree-arrow {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 16px;
  height: 16px;
  flex-shrink: 0;
  color: var(--text-dim);
  transition: transform 0.15s ease;
  transform: rotate(90deg);
}

.tree-arrow.collapsed {
  transform: rotate(0deg);
}

.tree-arrow-spacer {
  width: 16px;
  flex-shrink: 0;
}

/* Icons */
.tree-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;
  flex-shrink: 0;
  margin-right: 6px;
  color: var(--text-dim);
}

.tree-icon.icon-dir {
  color: #e2b55a;
}

.tree-icon.icon-open {
  color: #e8c76a;
}

/* Name */
.tree-name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 13px;
  line-height: 1.4;
}

.tree-row.dir .tree-name {
  font-weight: 500;
}

/* Badge */
.tree-badge {
  margin-left: auto;
  padding: 0 5px;
  font-size: 10px;
  color: var(--text-dim);
  background: var(--bg);
  border-radius: 8px;
  line-height: 16px;
  flex-shrink: 0;
}

/* Expand transition */
.expand-enter-active,
.expand-leave-active {
  overflow: hidden;
  transition: opacity 0.15s ease;
}

.expand-enter-from,
.expand-leave-to {
  opacity: 0;
}
</style>
