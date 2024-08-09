<script setup>

import { emit, listen } from '@tauri-apps/api/event';
// import { register, unregister, unregisterAll  } from '@tauri-apps/api/globalShortcut';
import { useRouter } from 'vue-router';
import { ref, nextTick, onMounted, onUnmounted } from "vue";


const router = useRouter();

onMounted(async () => {
  // await unregister('CommandOrControl+Enter');
  // await unregisterAll();
  // await register('CommandOrControl+Enter', () => {
  //   console.log('Shortcut triggered');
  // });

  //emits
  const unlisten_open_settings = await listen('open_settings', (event) => {
    router.push("/settings");
  });
  const unlisten_open_main = await listen('open_main', (event) => {
    router.push("/");
  });
  const unlisten_open_assistants = await listen('open_assistants', (event) => {
    router.push("/assistants");
  });
  const unlisten_open_ai_files = await listen('open_open_ai_files', (event) => {
    router.push("/open_ai_files");
  });
  const unlisten_open_samples = await listen('open_samples', (event) => {
    router.push("/samples");
  });
});
</script>

<template>
  <router-view></router-view>
</template>

<style></style>