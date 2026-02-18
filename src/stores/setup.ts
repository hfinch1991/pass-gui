import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { SetupStatus, GpgKey, StoreInfo } from "../types";

export const useSetupStore = defineStore("setup", () => {
  const currentStep = ref(0);
  const setupStatus = ref<SetupStatus | null>(null);
  const storeInfo = ref<StoreInfo | null>(null);
  const gpgKeys = ref<GpgKey[]>([]);
  const selectedKeyId = ref<string | null>(null);
  const gitEnabled = ref(false);
  const remoteUrl = ref("");
  const storeInitialized = ref(false);
  const useExistingStore = ref(false);
  const gitConfigured = ref(false);

  const depsLoading = ref(false);
  const depsError = ref<string | null>(null);
  const installLoading = ref(false);
  const installError = ref<string | null>(null);
  const keysLoading = ref(false);
  const keysError = ref<string | null>(null);
  const genKeyLoading = ref(false);
  const genKeyError = ref<string | null>(null);
  const initLoading = ref(false);
  const initError = ref<string | null>(null);
  const gitLoading = ref(false);
  const gitError = ref<string | null>(null);

  const canProceedFromStep = computed(() => {
    switch (currentStep.value) {
      case 0:
        return setupStatus.value?.dependencies_ok === true;
      case 1:
        return selectedKeyId.value !== null;
      case 2:
        return storeInitialized.value || useExistingStore.value;
      case 3:
        return true; // always skippable
      default:
        return false;
    }
  });

  async function checkStatus() {
    depsLoading.value = true;
    depsError.value = null;
    try {
      setupStatus.value = await invoke<SetupStatus>("check_setup_status");
      if (setupStatus.value.store_exists) {
        storeInitialized.value = true;
      }
    } catch (e: any) {
      depsError.value = typeof e === "string" ? e : e.message || "Failed to check status";
    } finally {
      depsLoading.value = false;
    }
  }

  async function loadStoreInfo() {
    try {
      storeInfo.value = await invoke<StoreInfo>("get_store_info");
      if (storeInfo.value.git_remote) {
        remoteUrl.value = storeInfo.value.git_remote;
        gitEnabled.value = true;
      }
    } catch {
      // not critical
    }
  }

  async function installDeps() {
    installLoading.value = true;
    installError.value = null;
    try {
      setupStatus.value = await invoke<SetupStatus>("install_dependencies");
    } catch (e: any) {
      installError.value = typeof e === "string" ? e : e.message || "Failed to install";
    } finally {
      installLoading.value = false;
    }
  }

  async function loadGpgKeys() {
    keysLoading.value = true;
    keysError.value = null;
    try {
      gpgKeys.value = await invoke<GpgKey[]>("list_gpg_keys");
      if (gpgKeys.value.length === 1) {
        selectedKeyId.value = gpgKeys.value[0].key_id;
      }
      // If store exists, pre-select the key matching the store's GPG ID
      if (storeInfo.value?.gpg_id) {
        const match = gpgKeys.value.find(
          (k) => k.key_id === storeInfo.value!.gpg_id || k.fingerprint.endsWith(storeInfo.value!.gpg_id!)
        );
        if (match) {
          selectedKeyId.value = match.key_id;
        }
      }
    } catch (e: any) {
      keysError.value = typeof e === "string" ? e : e.message || "Failed to list keys";
    } finally {
      keysLoading.value = false;
    }
  }

  async function generateKey(name: string, email: string, passphrase: string) {
    genKeyLoading.value = true;
    genKeyError.value = null;
    try {
      const key = await invoke<GpgKey>("generate_gpg_key", { name, email, passphrase });
      gpgKeys.value.push(key);
      selectedKeyId.value = key.key_id;
    } catch (e: any) {
      genKeyError.value = typeof e === "string" ? e : e.message || "Failed to generate key";
    } finally {
      genKeyLoading.value = false;
    }
  }

  async function initStore() {
    if (!selectedKeyId.value) return;
    initLoading.value = true;
    initError.value = null;
    try {
      await invoke<string>("init_password_store", { gpgId: selectedKeyId.value });
      storeInitialized.value = true;
    } catch (e: any) {
      initError.value = typeof e === "string" ? e : e.message || "Failed to init store";
    } finally {
      initLoading.value = false;
    }
  }

  function confirmExistingStore() {
    useExistingStore.value = true;
  }

  async function setupGit() {
    gitLoading.value = true;
    gitError.value = null;
    try {
      await invoke<string>("init_git_repo");
      if (remoteUrl.value.trim()) {
        await invoke<string>("add_git_remote", { url: remoteUrl.value.trim() });
      }
      gitConfigured.value = true;
      await loadStoreInfo();
    } catch (e: any) {
      gitError.value = typeof e === "string" ? e : e.message || "Failed to setup git";
    } finally {
      gitLoading.value = false;
    }
  }

  async function markComplete() {
    try {
      await invoke("mark_setup_complete");
    } catch {
      // not critical
    }
  }

  function nextStep() {
    if (currentStep.value < 3) {
      currentStep.value++;
    }
  }

  function prevStep() {
    if (currentStep.value > 0) {
      currentStep.value--;
    }
  }

  return {
    currentStep,
    setupStatus,
    storeInfo,
    gpgKeys,
    selectedKeyId,
    gitEnabled,
    remoteUrl,
    storeInitialized,
    useExistingStore,
    gitConfigured,
    depsLoading,
    depsError,
    installLoading,
    installError,
    keysLoading,
    keysError,
    genKeyLoading,
    genKeyError,
    initLoading,
    initError,
    gitLoading,
    gitError,
    canProceedFromStep,
    checkStatus,
    loadStoreInfo,
    installDeps,
    loadGpgKeys,
    generateKey,
    initStore,
    confirmExistingStore,
    setupGit,
    markComplete,
    nextStep,
    prevStep,
  };
});
