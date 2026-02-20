import { createRouter, createWebHistory } from 'vue-router';
import MainMenu from '../components/MainMenu.vue';
import ScriptSelector from '../components/ScriptSelector.vue';
import GameViewPage from '../views/GameViewPage.vue';

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
      component: GameViewPage,
    },
  ],
});

export default router;
