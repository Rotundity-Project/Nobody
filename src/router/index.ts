import { createRouter, createWebHistory } from 'vue-router';
import MainMenu from '../components/MainMenu.vue';
import ScriptSelector from '../components/ScriptSelector.vue';
import GameView from '../components/GameView.vue';

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'menu',
      component: MainMenu,
    },
    {
      path: '/script-select',
      name: 'script-select',
      component: ScriptSelector,
    },
    {
      path: '/game',
      name: 'game',
      component: GameView,
    },
  ],
});

export default router;
