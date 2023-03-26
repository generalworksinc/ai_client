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
const send_role = ref("user");
const tempareture = ref(0.9);
const template = ref("");
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
const add_template = () => {
    message.value += "\n" + template.value;
}
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
const translateToJp = () => {
    message.value = "translate to japanese below.\n" + message.value;
    sendMessageStream();
}
const translateToEn = () => {
    message.value = "translate to English below.\n" + message.value;
    sendMessageStream();
}
const sendMessageStream = () => {
    const userMessage = { 'role': send_role.value, 'content': message.value };
    all_messages.value.push(userMessage);
    now_messaging.value = "";
    message.value = '';

    invoke('send_message_and_callback_stream', {
        params: JSON.stringify({
            messages: all_messages.value,
            model: "gpt-3.5-turbo",
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

const ROLES = ['user', 'system'];
const TEMPLATES = [
    `If the question cannot be answered using the information provided answer with "I don't know"`,
    "Let's think logically, step by step. ",
     "First,", "Let's think about this logically.",
      "Let's solve this problem by splitting it into steps."]
</script>

<template>
    <div class="container">
        <div style="display: flex; justify-content: space-between;">
            <h3>chatGPT3.5</h3>
            <button @click="new_chat">new chat</button>
        </div>

        <div>click "send" or ctrl + enter to send message.<label v-for="role in ROLES" :key="'role_' + role">
                <input type="radio" v-model="send_role" :value="role" />{{ role }}
            </label></div>
        <div>tempareture: <input type="text" v-model="tempareture"><button @click="add_template">add
                template</button><select v-model="template">
                <option v-for="value in TEMPLATES" :value="value" :key="'template_' + value">{{ value }}</option>
            </select></div>
        <div style="display: flex; align-items: flex-end;">
            <textarea type="text" v-model="message" @keydown.ctrl.enter="sendMessageStream"
                style="height: 3rem; width: 80%;"></textarea>
            <!-- <button @click="sendMessage">send</button> -->
            <button @click="sendMessageStream">send</button>
            <button @click="translateToJp">translate to Jp</button>
            <button @click="translateToEn">translate to En</button>
        </div>

        <div id="article" style="overflow-y: scroll; max-height: 70vh;">
            <article v-for="(msg, ind) in all_messages" :key="'msg_' + ind" :style="ind > 0 ? 'margin-top: 2rem;' : ''">
                <div v-if="msg.role == 'user' || 'system'">
                    <div>
                        <span v-if="msg.role == 'user'">You</span>
                        <span v-if="msg.role == 'system'">System</span>
                    </div>
                    <div style="white-space:pre-wrap;">{{ msg.content }}</div>
                </div>
                <div v-else>
                    <div>
                        <span>chatGPT</span>
                    </div>
                    <p v-if="disp_raw_text_indexes.includes(ind)" style="white-space:pre-wrap;">{{ msg.content }}</p>
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
}</style>
