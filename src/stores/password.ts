import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { TreeNode, EntryFields, TotpCode } from "../types";

export const usePasswordStore = defineStore("password", () => {
  const tree = ref<TreeNode[]>([]);
  const selectedPath = ref<string | null>(null);
  const detailContent = ref<string | null>(null);
  const entryFields = ref<EntryFields | null>(null);
  const totpCode = ref<TotpCode | null>(null);
  const detailLoading = ref(false);
  const searchQuery = ref("");
  const checkedPaths = ref<Set<string>>(new Set());
  const loading = ref(false);
  const error = ref<string | null>(null);
  const showRaw = ref(false);

  const needsPassphrase = ref(false);
  const cachedPassphrase = ref<string | null>(null);
  const passphraseError = ref<string | null>(null);
  const pendingAction = ref<(() => Promise<void>) | null>(null);

  function filterTree(nodes: TreeNode[], query: string): TreeNode[] {
    if (!query) return nodes;
    const lower = query.toLowerCase();
    return nodes
      .map((node) => {
        // If the path itself contains the query, return the whole node (including all children)
        if (node.path.toLowerCase().includes(lower)) {
          return node;
        }
        
        // Otherwise, if it's a directory, check if any of its children match
        if (node.is_dir) {
          const filteredChildren = filterTree(node.children, query);
          if (filteredChildren.length > 0) {
            return { ...node, children: filteredChildren };
          }
        }
        
        return null;
      })
      .filter((n): n is TreeNode => n !== null);
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

  async function showPassword(path: string, passphrase?: string) {
    selectedPath.value = path;
    detailContent.value = null;
    entryFields.value = null;
    totpCode.value = null;
    detailLoading.value = true;
    showRaw.value = false;
    passphraseError.value = null;

    const pw = passphrase || cachedPassphrase.value;

    try {
      const [raw, fields] = await Promise.all([
        invoke<string>("show_password", { path, passphrase: pw }),
        invoke<EntryFields>("get_entry_fields", { path, passphrase: pw }),
      ]);
      detailContent.value = raw;
      entryFields.value = fields;
      
      if (pw) {
        cachedPassphrase.value = pw;
      }

      // Load TOTP if available
      if (fields.totp) {
        refreshTotp(path, pw);
      }
    } catch (e: any) {
      const err = typeof e === "string" ? e : e.message || "Decryption failed";
      if (err.includes("GPG decryption failed") || err.includes("bad passphrase") || err.includes("decryption failed")) {
        needsPassphrase.value = true;
        passphraseError.value = passphrase ? "Incorrect passphrase" : null;
        pendingAction.value = () => showPassword(path);
      } else {
        detailContent.value = `Error: ${err}`;
      }
    } finally {
      detailLoading.value = false;
    }
  }

  async function submitPassphrase(pw: string) {
    if (pendingAction.value) {
      const action = pendingAction.value;
      pendingAction.value = null;
      needsPassphrase.value = false;
      
      // Try again with the provided passphrase
      if (selectedPath.value) {
        await showPassword(selectedPath.value, pw);
      }
    }
  }

  function cancelPassphrase() {
    needsPassphrase.value = false;
    passphraseError.value = null;
    pendingAction.value = null;
    detailLoading.value = false;
  }

  let totpInterval: ReturnType<typeof setInterval> | null = null;

  async function refreshTotp(path: string, passphrase?: string | null) {
    if (totpInterval) {
      clearInterval(totpInterval);
      totpInterval = null;
    }
    const pw = passphrase || cachedPassphrase.value;
    try {
      totpCode.value = await invoke<TotpCode>("get_totp_code", { path, passphrase: pw });
      // Auto-refresh TOTP
      totpInterval = setInterval(async () => {
        if (selectedPath.value !== path) {
          if (totpInterval) clearInterval(totpInterval);
          return;
        }
        try {
          totpCode.value = await invoke<TotpCode>("get_totp_code", { path, passphrase: pw });
        } catch {
          totpCode.value = null;
          if (totpInterval) clearInterval(totpInterval);
        }
      }, 1000);
    } catch {
      totpCode.value = null;
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
      entryFields.value = null;
      totpCode.value = null;
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
      entryFields.value = null;
      totpCode.value = null;
    }
    checkedPaths.value = new Set();
    await loadTree();
  }

  return {
    tree,
    selectedPath,
    detailContent,
    entryFields,
    totpCode,
    detailLoading,
    searchQuery,
    checkedPaths,
    loading,
    error,
    showRaw,
    needsPassphrase,
    cachedPassphrase,
    passphraseError,
    pendingAction,
    filteredTree,
    loadTree,
    showPassword,
    submitPassphrase,
    cancelPassphrase,
    refreshTotp,
    toggleChecked,
    clearChecked,
    toggleAllInNode,
    deleteSingle,
    deleteSelected,
  };
});
