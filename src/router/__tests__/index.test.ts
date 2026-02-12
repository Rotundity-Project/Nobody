import { describe, expect, it } from 'vitest';
import router from '../index';

describe('router', () => {
  it('registers expected routes', () => {
    const routes = router.getRoutes();
    const paths = routes.map((route) => route.path);
    expect(paths).toContain('/');
    expect(paths).toContain('/script-select');
    expect(paths).toContain('/game');
  });

  it('navigates to game route', async () => {
    await router.push('/game');
    await router.isReady();
    expect(router.currentRoute.value.path).toBe('/game');
  });
});
