<script setup lang="ts">
import { useI18n } from 'vue-i18n';

const { t } = useI18n()

/*
const sidebar_active = ref(0)
const theme = ref("dark")
*/

const items = [
  {
    label: t("home"),
    to: "/",
  },
  {
    label: t("saves"),
    to: "/saves",
  },
  {
    label: t("settings"),
    to: "/settings",
  }
];

</script>

<template>
  <div id="sidebar" class="py-2 rounded-xl bg-black/5 dark:bg-white/5">
    <div class="p-2 text-center">TODO</div>
    <div class="my-2 p-2 text-center hover:bg-black/5 dark:hover:bg-white/5" v-for="item in items">
      <router-link :to="item.to">
        {{ item.label }}
      </router-link>
    </div>
  </div>
  <div id="main" class="ml-2 p-6 rounded-xl bg-black/5 dark:bg-white/5">
    <RouterView />
  </div>
</template>
<style scoped>
#sidebar {
  flex: 0 0 auto;
}

#main {
  flex: 1 0 auto;
  width: 0;
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
