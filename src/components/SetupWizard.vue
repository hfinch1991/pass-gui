<script setup lang="ts">
import { onMounted } from "vue";
import { useSetupStore } from "../stores/setup";
import StepDependencies from "./StepDependencies.vue";
import StepGpgKey from "./StepGpgKey.vue";
import StepInitStore from "./StepInitStore.vue";
import StepGitRemote from "./StepGitRemote.vue";

const emit = defineEmits<{ complete: [] }>();
const setup = useSetupStore();

const steps = [
  { label: "Dependencies", component: StepDependencies },
  { label: "GPG Key", component: StepGpgKey },
  { label: "Init Store", component: StepInitStore },
  { label: "Git Remote", component: StepGitRemote },
];

onMounted(() => {
  setup.loadStoreInfo();
});

async function finish() {
  await setup.markComplete();
  emit("complete");
}
</script>

<template>
  <div class="wizard">
    <div class="wizard-header">
      <h2>Setup Wizard</h2>
      <p>Let's get your password store ready.</p>
    </div>

    <div class="step-indicators">
      <div
        v-for="(step, i) in steps"
        :key="i"
        class="step-indicator"
        :class="{
          active: i === setup.currentStep,
          completed: i < setup.currentStep,
        }"
      >
        <div class="dot">{{ i < setup.currentStep ? "\u2713" : i + 1 }}</div>
        <span class="step-label">{{ step.label }}</span>
      </div>
    </div>

    <div class="step-content">
      <KeepAlive>
        <component :is="steps[setup.currentStep].component" />
      </KeepAlive>
    </div>

    <div class="wizard-nav">
      <button
        v-if="setup.currentStep > 0"
        @click="setup.prevStep()"
      >
        Back
      </button>
      <div class="spacer" />
      <button
        v-if="setup.currentStep < 3"
        class="primary"
        :disabled="!setup.canProceedFromStep"
        @click="setup.nextStep()"
      >
        Next
      </button>
      <button
        v-if="setup.currentStep === 3"
        class="primary"
        @click="finish()"
      >
        Finish
      </button>
    </div>
  </div>
</template>

<style scoped>
.wizard {
  max-width: 540px;
  margin: 0 auto;
  padding: 40px 24px;
  display: flex;
  flex-direction: column;
  gap: 24px;
  min-height: 100vh;
}
.wizard-header {
  text-align: center;
}
.wizard-header h2 {
  font-size: 20px;
  margin-bottom: 4px;
}
.wizard-header p {
  color: var(--text-dim);
  font-size: 13px;
}
.step-indicators {
  display: flex;
  justify-content: center;
  gap: 24px;
}
.step-indicator {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
}
.dot {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  font-weight: 600;
  background: var(--bg-hover);
  color: var(--text-dim);
  border: 2px solid var(--border);
}
.step-indicator.active .dot {
  background: var(--accent);
  color: #1e1e2e;
  border-color: var(--accent);
}
.step-indicator.completed .dot {
  background: var(--success);
  color: #1e1e2e;
  border-color: var(--success);
}
.step-label {
  font-size: 11px;
  color: var(--text-dim);
}
.step-indicator.active .step-label {
  color: var(--text);
  font-weight: 600;
}
.step-content {
  background: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 20px;
  min-height: 200px;
}
.wizard-nav {
  display: flex;
  align-items: center;
  gap: 8px;
}
.spacer {
  flex: 1;
}
</style>
