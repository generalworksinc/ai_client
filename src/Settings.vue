<script setup>
// This starter template is using Vue 3 <script setup> SFCs
// Check out https://vuejs.org/api/sfc-script-setup.html#script-setup
import { invoke, convertFileSrc } from '@tauri-apps/api/tauri'
import { useRouter } from 'vue-router';
import { ref, onMounted, onUnmounted  } from "vue";
const router = useRouter();

const api_key = ref("");
const saving_directory = ref("");

//methods
const cancel = () => {
    // router.push({ name: 'qa', params: { api_key: api_key.value } });
    router.push("/");
}
const saveConfig = () => {
    invoke('set_api_key', { apiKey: api_key.value, savingDirectory: saving_directory.value }).then(async res => {
        router.push("/");
    });
}
onMounted(async () => {
    invoke('get_api_key', {}).then(async res => {
        const resJson = JSON.parse(res);
        console.log('resJson:', resJson);
        api_key.value = resJson.apiKey;
        saving_directory.value = resJson.savingDirectory;
    });
});
</script>

<template>
    <div class="container">
        <h1>AI Client</h1>

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
        <div>
            <div>
                <label for="save_directory" style="width: 100px; vertical-align:bottom; white-space:nowrap;">Saving Directory: </label>
            </div>
            <input id="saving_directory" type="text" v-model="saving_directory" style="width: 100%;">
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
