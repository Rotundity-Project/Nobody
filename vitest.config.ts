import { configDefaults, defineConfig } from 'vitest/config';
import vue from '@vitejs/plugin-vue';

export default defineConfig({
  plugins: [vue()],
  test: {
    environment: 'jsdom',
    globals: true,
    clearMocks: true,
    // Frontend tests live under src/; keeping discovery here avoids .review worktrees
    // and other local review copies being picked up as part of the current repo run.
    include: ['src/**/*.{test,spec}.{ts,tsx}'],
    exclude: [
      ...configDefaults.exclude,
      '.review/**',
    ],
  },
});
