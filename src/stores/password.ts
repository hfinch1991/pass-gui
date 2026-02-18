import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { TreeNode } from "../types";

export const usePasswordStore = defineStore("password", () => {
  const tree = ref<TreeNode[]>([]);
  const selectedPath = ref<string | null>(null);
  const detailContent = ref<string | null>(null);
  const detailLoading = ref(false);
  const searchQuery = ref("");
  const checkedPaths = ref<Set<string>>(new Set());
  const loading = ref(false);
  const error = ref<string | null>(null);

  function filterTree(nodes: TreeNode[], query: string): TreeNode[] {
    if (!query) return nodes;
    const lower = query.toLowerCase();
    return nodes
      .map((node) => {
        if (node.is_dir) {
          const filtered = filterTree(node.children, query);
          if (filtered.length > 0) {
            return { ...node, children: filtered };
          }
          return null;
        }
        return node.name.toLowerCase().includes(lower) ? node : null;
      })
      .filter(Boolean) as TreeNode[];
  }

  const filteredTree = computed(() => filterTree(tree.value, searchQuery.value));

  async function loadTree() {
    loading.value = true;
    error.value = null;
    try {
      tree.value = await invoke<TreeNode[]>("list_store");
    } catch (e: any) {
      error.value = typeof e === "string" ? e : e.message || "Failed to load store";
    } finally {
      loading.value = false;
    }
  }

  async function showPassword(path: string) {
    selectedPath.value = path;
    detailContent.value = null;
    detailLoading.value = true;
    try {
      detailContent.value = await invoke<string>("show_password", { path });
    } catch (e: any) {
      detailContent.value = `Error: ${typeof e === "string" ? e : e.message}`;
    } finally {
      detailLoading.value = false;
    }
  }

  function toggleChecked(path: string) {
    const s = new Set(checkedPaths.value);
    if (s.has(path)) {
      s.delete(path);
    } else {
      s.add(path);
    }
    checkedPaths.value = s;
  }

  function clearChecked() {
    checkedPaths.value = new Set();
  }

  function toggleAllInNode(node: TreeNode, checked: boolean) {
    const s = new Set(checkedPaths.value);
    function walk(n: TreeNode) {
      if (n.is_dir) {
        n.children.forEach(walk);
      } else {
        if (checked) s.add(n.path);
        else s.delete(n.path);
      }
    }
    walk(node);
    checkedPaths.value = s;
  }

  async function deleteSingle(path: string) {
    await invoke("remove_password", { path });
    if (selectedPath.value === path) {
      selectedPath.value = null;
      detailContent.value = null;
    }
    checkedPaths.value.delete(path);
    await loadTree();
  }

  async function deleteSelected() {
    const paths = Array.from(checkedPaths.value);
    if (paths.length === 0) return;
    await invoke("remove_batch", { paths });
    if (selectedPath.value && checkedPaths.value.has(selectedPath.value)) {
      selectedPath.value = null;
      detailContent.value = null;
    }
    checkedPaths.value = new Set();
    await loadTree();
  }

  return {
    tree,
    selectedPath,
    detailContent,
    detailLoading,
    searchQuery,
    checkedPaths,
    loading,
    error,
    filteredTree,
    loadTree,
    showPassword,
    toggleChecked,
    clearChecked,
    toggleAllInNode,
    deleteSingle,
    deleteSelected,
  };
});
