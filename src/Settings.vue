<script setup>
// This starter template is using Vue 3 <script setup> SFCs
// Check out https://vuejs.org/api/sfc-script-setup.html#script-setup
import Greet from "./components/Greet.vue";
import { onMounted, onUnmounted } from '@vue/runtime-core';
import { invoke, convertFileSrc } from '@tauri-apps/api/tauri'
import { useRouter } from 'vue-router';
import { ref } from "vue";
const router = useRouter();

const api_key = ref("");

//methods
const cancel = () => {
    // router.push({ name: 'qa', params: { api_key: api_key.value } });
    router.push("/");
}
const saveConfig = () => {
    invoke('set_api_key', { value: api_key.value }).then(async res => {
        router.push("/");
    });
}
onMounted(async () => {
    invoke('get_api_key', {}).then(async res => {
        console.log('response.', res);
        api_key.value = res;
    });
});
</script>

<template>
    <div class="container">
        <h1>AI base Desktop</h1>

        <div class="row">
            <!-- <a href="https://vitejs.dev" target="_blank">
                                                                                    <img src="/vite.svg" class="logo vite" alt="Vite logo" />
                                                                                </a> -->
            <a href="https://tauri.app" target="_blank">
                <img src="/tauri.svg" class="logo tauri" alt="Tauri logo" />
            </a>
            <a href="https://chat.openai.com/" target="_blank">
                <img src="./assets/chatgpt.png" class="logo chatgpt" alt="ChatGPT logo" style="width: auto" />
            </a>
        </div>

        <p style="text-align: center;">This application is built by tauri / ChatGPT API.</p>
        <div>
            <div>
                <label for="api_key" style="width: 100px; vertical-align:bottom; white-space:nowrap;">ChatGPT API
                    Key:</label>
            </div>
            <input id="api_key" type="text" v-model="api_key" style="width: 100%;">
        </div>
        <div style="margin-top: 2rem; display:flex; justify-content: space-evenly; text-align: center;">
            <button @click="saveConfig">Save</button>
            <button @click="cancel">Cancel</button>
        </div>
        <!-- <Greet /> -->
    </div>
</template>

<style scoped>
.logo.vite:hover {
    filter: drop-shadow(0 0 2em #747bff);
}

.logo.vue:hover {
    filter: drop-shadow(0 0 2em #249b73);
}

.logo.chatgpt:hover {
    filter: drop-shadow(0 0 2em #777);
}
</style>
