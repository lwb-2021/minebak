<script setup lang="ts">
import { useI18n } from 'vue-i18n';

import { Sidebar, SidebarItem } from 'vant';
const { t } = useI18n()

</script>

<template>
  <Sidebar id="sidebar">
    <SidebarItem :title="t('home')" to="/" />
    <SidebarItem :title="t('saves')" to="/saves" />
    <SidebarItem :title="t('settings')" to="/settings" />
  </Sidebar>
  <RouterView id="main" />
</template>
<style scoped>
#sidebar {
  flex: 0 0 auto;
}

#main {
  flex: 1 0 auto;
  padding: 0 8px;
}
</style>

<i18n lang="json">{
  "en-us": {
    "home": "Home",
    "saves": "Saves",
    "settings": "Settings"
  },
  "zh-cn": {
    "home": "主页",
    "saves": "存档列表",
    "settings": "设置"
  }
}</i18n>
