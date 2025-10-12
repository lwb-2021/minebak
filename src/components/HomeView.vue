<template>
  <h1>{{ t("welcome-to") + " MineBak" }}</h1>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';

const { t } = useI18n()
</script>

<style lang="css" scoped>
h1 {
  text-align: center;
}
</style>
<i18n lang="json">{
  "zh-cn": {
    "welcome-to": "欢迎使用"
  },
  "en-us": {
    "welcome-to": "Welcome to"
  }
}</i18n>
