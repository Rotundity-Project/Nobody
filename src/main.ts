import { createApp } from 'vue'
import { createPinia } from 'pinia'
import router from './router'
import App from './App.vue'
import './styles.css'

const renderStartupError = (error: unknown) => {
  const message = error instanceof Error ? error.stack || error.message : String(error)
  const appRoot = document.querySelector<HTMLDivElement>('#app')
  if (!appRoot) return
  appRoot.innerHTML = `
    <main style="min-height:100vh;padding:24px;font-family:system-ui,-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif;background:#fff7ed;color:#1f2933;">
      <h1 style="margin:0 0 12px;font-size:20px;">Nobody 启动失败</h1>
      <p style="margin:0 0 16px;line-height:1.7;">前端脚本运行时出错，已拦截白屏。请把下面错误信息发给开发者。</p>
      <pre style="white-space:pre-wrap;overflow:auto;padding:16px;border:1px solid #d8b99a;border-radius:8px;background:#fffaf2;color:#7c2d12;">${message
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')}</pre>
    </main>
  `
}

window.addEventListener('error', (event) => {
  renderStartupError(event.error ?? event.message)
})

window.addEventListener('unhandledrejection', (event) => {
  renderStartupError(event.reason)
})

try {
  const app = createApp(App)
  const pinia = createPinia()

  app.config.errorHandler = (error) => {
    renderStartupError(error)
  }

  app.use(pinia)
  app.use(router)
  app.mount('#app')
  window.dispatchEvent(new CustomEvent('nobody:app-mounted'))
} catch (error) {
  renderStartupError(error)
}
