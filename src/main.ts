import { createApp } from 'vue'

import 'vant/lib/index.css'
import './style.css'

import App from './App.vue'

import { createMemoryHistory, createRouter } from 'vue-router'
import { routes } from './routes'
import { createI18n } from 'vue-i18n'

const router = createRouter({
  history: createMemoryHistory(),
  routes: routes
})

const i18n = createI18n({
  legacy: false, // you must set `false`, to use Composition API
  locale: 'zh-cn',
  fallbackLocale: 'en-us',
})



createApp(App).use(router).use(i18n).mount('#app')
