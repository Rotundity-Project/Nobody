<template>
  <div class="min-h-screen flex items-center justify-center bg-gradient-to-br from-slate-900 via-purple-900 to-slate-900 p-8">
    <div class="max-w-2xl w-full bg-slate-800 rounded-lg shadow-2xl p-8">
      <h2 class="text-3xl font-bold text-white mb-6">Select Script Type</h2>
      
      <div class="space-y-4">
        <div
          v-for="scriptType in scriptTypes"
          :key="scriptType.type"
          class="p-6 rounded-lg border-2 transition-all duration-200"
          :class="[
            scriptType.available 
              ? 'border-purple-500 bg-slate-700 hover:bg-slate-600 cursor-pointer' 
              : 'border-gray-600 bg-slate-800 opacity-50 cursor-not-allowed'
          ]"
          @click="scriptType.available && selectScriptType(scriptType.type)"
        >
          <div class="flex items-center justify-between">
            <div>
              <h3 class="text-xl font-semibold text-white mb-2">
                {{ scriptType.title }}
              </h3>
              <p class="text-gray-300">{{ scriptType.description }}</p>
            </div>
            <div v-if="!scriptType.available" class="text-gray-500 text-sm">
              Coming Soon
            </div>
          </div>
        </div>
      </div>

      <div v-if="isLoading" class="mt-6 text-center">
        <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-purple-500"></div>
        <p class="text-gray-300 mt-2">Loading...</p>
      </div>

      <div v-if="error" class="mt-6 p-4 bg-red-900 bg-opacity-50 border border-red-500 rounded-lg">
        <p class="text-red-200">{{ error }}</p>
      </div>

      <button
        @click="handleBack"
        class="mt-6 px-6 py-2 bg-gray-600 hover:bg-gray-700 text-white rounded-lg transition-colors duration-200"
      >
        Back
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useRouter } from 'vue-router';
import { open } from '@tauri-apps/plugin-dialog';
import type { ScriptType } from '../types/game';

const router = useRouter();
const isLoading = ref(false);
const error = ref<string | null>(null);

interface ScriptTypeOption {
  type: ScriptType;
  title: string;
  description: string;
  available: boolean;
}

const scriptTypes = ref<ScriptTypeOption[]>([
  {
    type: 'custom' as ScriptType,
    title: 'Custom Script',
    description: 'Load a custom JSON script file',
    available: true,
  },
  {
    type: 'random_generated' as ScriptType,
    title: 'Random Generated',
    description: 'Generate a random cultivation world using AI',
    available: false,
  },
  {
    type: 'existing_novel' as ScriptType,
    title: 'Existing Novel',
    description: 'Import script from existing cultivation novel',
    available: false,
  },
]);

const selectScriptType = async (type: ScriptType) => {
  error.value = null;

  if (type === 'custom') {
    await loadCustomScript();
  } else {
    error.value = 'This feature is not yet implemented';
  }
};

const loadCustomScript = async () => {
  try {
    isLoading.value = true;
    
    const selected = await open({
      multiple: false,
      filters: [{
        name: 'JSON',
        extensions: ['json']
      }]
    });

    if (selected) {
      // TODO: Load script and initialize game
      // For now, just navigate to game view
      router.push('/game');
    }
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Failed to load script';
  } finally {
    isLoading.value = false;
  }
};

const handleBack = () => {
  router.push('/');
};
</script>
