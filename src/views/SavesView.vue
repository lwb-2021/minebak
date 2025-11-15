<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';
import { useI18n } from 'vue-i18n';
import InstanceCard from '../components/saves/InstanceCard.vue';
import { onMounted, ref } from 'vue';

import { open } from "@tauri-apps/plugin-dialog";


const { t } = useI18n()

const data = ref([])
const loading = ref(true)

const path = ref("")
const instance_type = ref(0)

const on_add_instance_submit = async () => {
  return await invoke("add_root",
    {
      path: path.value,
      instanceType: Number(instance_type.value)
    }
  )
}

onMounted(async () => {
  data.value = await invoke("list_instances")
  loading.value = false
})
</script>
<template>
  <div class="grid grid-cols-4 gap-4">
    <input class="col-span-3" :placeholder="t('search')"></input>
    <button class="" @click="() => this.$refs.dialog_add_instance.showModal()">
      添加实例
    </button>
  </div>
  <dialog ref="dialog_add_instance">
    <form method="dialog" class="p-4">
      <div class="my-2 mx-4 text-lg">
        输入实例的上一级路径（一般为.minecraft文件夹）
      </div>
      <div class="my-1 flex flex-col gap-2">
        <div class="flex gap-2">
          <label class="w-20 py-1" for="instance_type">{{ t("instance-type") }}</label>
          <select id="instance_type" class="grow" v-model="instance_type">
            <option value="0">{{ t("instance-normal") }}</option>
            <option value="1">{{ t("instance-isolated") }}</option>
            <option value="2">MultiMC</option>

          </select>
        </div>
        <div class="flex gap-2">

          <label class="w-20 py-1" for="path">{{ t("instance-parent") }}</label>
          <input id="path" class="grow" v-model="path" />
          <button type="button" @click="async () => {
            // @ts-ignore
            path = await open({
              multiple: false,
              directory: true,
            });
          }">{{ t("browse") }}</button>
        </div>
      </div>
      <div class="flex my-2 gap-4">
        <button class="shrink w-24" type="button">填充默认</button>
        <div class="grow" />
        <button class="shrink w-24" @click="on_add_instance_submit">{{ t("confirm") }}</button>
        <button class="shrink w-24">{{ t("cancel") }}</button>
      </div>
    </form>
  </dialog>

  <div v-if="loading">Loading</div>
  <div v-else class="grid grid-cols-3 gap-4 py-4">
    <InstanceCard v-for="(value, name) in data" :name :meta="value" />
  </div>
</template>

<i18n lang="json">{
  "zh-cn": {
    "add-instances": "添加实例",
    "search": "搜索",
    "confirm": "确定",
    "cancel": "取消",
    "browse": "浏览",
    "dialog-add-instance-promot": "请输入实例的上一级路径（通常为.minecraft）",
    "instance-type": "实例类型",
    "instance-normal": "普通",
    "instance-isolated": "版本隔离",
    "instance-parent": "实例父路径"
  },
  "en-us": {
    "add-instances": "Add instances",
    "search": "Search",
    "confirm": "Confirm",
    "cancel": "Cancel",
    "browse": "浏览",
    "dialog-add-instance-prompt": "Please input the parent path of the instance (Usually .minecraft)",
    "instance-type": "Instance Type",
    "instance-normal": "Normal",
    "instance-isolated": "Version isolated",
    "instance-parent": "Instance Parent Path"
  }
}</i18n>
