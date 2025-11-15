<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
const props = defineProps({
  name: String,
  meta: Object,
})
</script>
<template>
  <div class="p-4 bg-black/2 dark:bg-white/5 rounded-xl text-pretty h-full flex flex-col">
    <h2 class="text-xl overflow-hidden text-ellipsis"> {{ props.name }} </h2>
    <div class="grow"> {{ props.meta }} </div>
    <div class="flex flex-row-reverse">
      <button class="operations" @click="async () => await invoke('run_instance_backup', { name: props.name })">
        <span class="icon-[dashicons--backup]" />
      </button>
    </div>
  </div>
</template>
<style lang="css" scoped>
@reference "../../style.css";

.operations {
  @apply bg-transparent p-2;

  @variant hover {
    @apply bg-black/5;

    @variant dark {
      @apply bg-white/5;
    }
  }

}
</style>
