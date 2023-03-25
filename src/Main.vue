<script setup>
// This starter template is using Vue 3 <script setup> SFCs
// Check out https://vuejs.org/api/sfc-script-setup.html#script-setup
import Greet from "./components/Greet.vue";
import { invoke, convertFileSrc } from '@tauri-apps/api/tauri'
import { emit, listen } from '@tauri-apps/api/event';
import { useRouter } from 'vue-router';
import { ref, nextTick } from "vue";
import { onMounted, onUnmounted } from '@vue/runtime-core';
const router = useRouter();

const message = ref("");
const all_messages = ref([]);
const all_messages_raw = ref([]);
const now_messaging = ref("");
const is_thinking = ref(false);
const disp_raw_text_indexes = ref([]);
let articleDom = null;

let unlisten_stream_chunk = null;
let unlisten_finish_chunks = null;
let unlisten_stream_error = null;

onUnmounted(async () => {
    if (unlisten_stream_chunk) {
        unlisten_stream_chunk();
    }
    if (unlisten_finish_chunks) {
        unlisten_finish_chunks();
    }
    if (unlisten_stream_error) {
        unlisten_stream_error();
    }
});

onMounted(async () => {
    articleDom = document.getElementById('article');
    //emits
    unlisten_stream_error = await listen('stream_error', (event) => {
        is_thinking.value = false;
        const errorObj = JSON.parse(event.payload);
        now_messaging.value = `<h3>${errorObj['type']}</h3><p>${errorObj['message']}</p>`;
        nextTick(() => {
            if (articleDom) {
                articleDom.scrollTo(0, articleDom.scrollHeight);
            }
        });
    });
    unlisten_stream_chunk = await listen('stream_chunk', (event) => {
        is_thinking.value = false;
        console.log('unlisten_finish_chunks called event.', event);
        now_messaging.value = event.payload;
        nextTick(() => {
            if (articleDom) {
                articleDom.scrollTo(0, articleDom.scrollHeight);
            }
        });
    });
    unlisten_finish_chunks = await listen('finish_chunks', (event) => {
        // is_thinking.value = false;
        // console.log('unlisten_finish_chunks called event.', event);
        // now_messaging.value = event.payload;

        is_thinking.value = false;
        if (now_messaging.value) {
            const lastAssistanceMessage = { 'role': 'assistant', 'content': event.payload, 'content_html': now_messaging.value };
            all_messages.value.push(lastAssistanceMessage);
            now_messaging.value = "";
        }
        nextTick(() => {
            if (articleDom) {
                articleDom.scrollTo(0, articleDom.scrollHeight);
            }
        });
    });

});

//methods
const new_chat = () => {
    window.location.reload();
}
const toggleDisplay = (index) => {
    const ind = disp_raw_text_indexes.value.indexOf(index);
    if (ind >= 0) {
        disp_raw_text_indexes.value.splice(ind, 1);
    } else {
        disp_raw_text_indexes.value.push(index);
    }

}
const sendMessageStream = () => {
    const userMessage = { 'role': 'user', 'content': message.value };
    all_messages.value.push(userMessage);
    now_messaging.value = "";
    message.value = '';

    invoke('send_message_and_callback_stream', {
        message: JSON.stringify({
            messages: all_messages.value,
            model: "",
            temperature: 0.9,
            max_tokens: 1024,
        })
    }).then(async res => {
        console.log('response.', res);
    });
    nextTick(() => {
        if (articleDom) {
            articleDom.scrollTo(0, articleDom.scrollHeight);
        }
    });
}
</script>

<template>
    <div class="container">
        <div style="display: flex; justify-content: space-between;">
            <h3>chatGPT3.5</h3>
            <button @click="new_chat">new chat</button>
        </div>

        <div>click "send" or ctrl + enter to send message.</div>
        <div style="display: flex; align-items: flex-end;">
            <textarea type="text" v-model="message" @keyup.ctrl.enter="sendMessageStream"
                style="height: 3rem; width: 80%;" />
            <!-- <button @click="sendMessage">send</button> -->
            <button @click="sendMessageStream">send</button>
        </div>

        <div id="article" style="overflow-y: scroll; max-height: 70vh;">
            <article v-for="(msg, ind) in all_messages" :key="'msg_' + ind" :style="ind > 0 ? 'margin-top: 2rem;' : ''">
                <div v-if="msg.role == 'user'">
                    <div>
                        <span>You</span>
                    </div>
                    <div style="white-space:pre;">{{ msg.content }}</div>
                </div>
                <div v-else>
                    <div>
                        <span>chatGPT</span>
                    </div>
                    <p v-if="disp_raw_text_indexes.includes(ind)" style="white-space:pre;">{{ msg.content }}</p>
                    <div v-else v-html="msg.content_html || msg.content"></div>
                    <button v-if="msg.content_html.replace('<p>', '').replace('</p>', '') != msg.content"
                        @click="toggleDisplay(ind)">
                        <span v-if="disp_raw_text_indexes.includes(ind)">display formatted text</span><span v-else>display
                            raw text</span>
                    </button>
                </div>
            </article>
            <article v-show="is_thinking || now_messaging" style="margin-top: 2rem;">
                <div><span>chatGPT</span></div>
                <div v-if="now_messaging" v-html="now_messaging"></div>
                <p v-else>I'm thinking...</p>
            </article>
        </div>
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
