<template>
  <div id="app">
    <MainMenu 
      v-if="currentView === 'menu'"
      @new-game="showScriptSelector"
      @load-game="handleLoadGame"
      @settings="handleSettings"
    />
    
    <ScriptSelector
      v-else-if="currentView === 'script-selector'"
      @script-selected="handleScriptSelected"
      @back="showMainMenu"
    />

    <div v-else-if="currentView === 'game'" class="min-h-screen bg-slate-900 text-white p-8">
      <div class="max-w-4xl mx-auto">
        <h2 class="text-2xl font-bold mb-4">游戏界面</h2>
        <p class="text-gray-300">游戏界面将在后续任务中实现</p>
        <button
          @click="showMainMenu"
          class="mt-4 px-6 py-2 bg-gray-600 hover:bg-gray-700 rounded-lg transition-colors"
        >
          返回主菜单
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import MainMenu from './components/MainMenu.vue';
import ScriptSelector from './components/ScriptSelector.vue';
import type { Script } from './types/game';

type View = 'menu' | 'script-selector' | 'game';

const currentView = ref<View>('menu');

const showMainMenu = () => {
  currentView.value = 'menu';
};

const showScriptSelector = () => {
  currentView.value = 'script-selector';
};

const handleScriptSelected = (script: Script) => {
  console.log('Script selected:', script);
  currentView.value = 'game';
};

const handleLoadGame = () => {
  console.log('Load game clicked');
};

const handleSettings = () => {
  console.log('Settings clicked');
};
</script>

<style>
#app {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}
</style>
