<script setup>
// This starter template is using Vue 3 <script setup> SFCs
// Check out https://vuejs.org/api/sfc-script-setup.html#script-setup
import Greet from "./components/Greet.vue";
import { invoke, convertFileSrc } from '@tauri-apps/api/tauri'
import { emit, listen } from '@tauri-apps/api/event';
import { useRouter } from 'vue-router';
import { ref, nextTick, onMounted, onUnmounted, computed } from "vue";
import { Multipane, MultipaneResizer } from './lib/multipane';
import { v4 as uuidv4 } from 'uuid';

import { AI_MODELS } from './constants';

const router = useRouter();

const message = ref("");
const assistant_name = ref("");
const instructions = ref("");


const now_messaging = ref("");
let now_messaging_raw = "";
const is_thinking = ref(false);
const disp_raw_text_indexes = ref([]);
const send_role = ref("user");
const tempareture = ref(0.9);
const template = ref("");
const ai_name = ref("gpt-4o-mini");
const search_word = ref("");
const errorMsg = ref("");
const lastWaitingMessageId = ref("");
const timeoutSec = ref(180);

const titleList = ref([]);
const searchResultList = ref([]);

let articleDom = null;

let unlisten_stream_chunk = null;
let unlisten_finish_chunks = null;
let unlisten_stream_error = null;
let unlisten_timeout_stream = null;

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
    if (unlisten_timeout_stream) {
        unlisten_timeout_stream();
    }
});

onMounted(async () => {
    articleDom = document.getElementById('article');
    // //emits
    // unlisten_stream_error = await listen('stream_error', (event) => {
    //     is_thinking.value = false;
    //     const errorObj = JSON.parse(event.payload);
    //     now_messaging.value = `<h3>${errorObj['type']}</h3><p>${errorObj['message']}</p>`;
    //     nextTick(() => {
    //         if (articleDom) {
    //             articleDom.scrollTo(0, articleDom.scrollHeight);
    //         }
    //     });
    // });
    // unlisten_stream_chunk = await listen('stream_chunk', (event) => {
    //     console.log('streamdata:', event.payload);
    //     const payload = event.payload;

    //     // is_thinking.value = false;
    //     if (lastWaitingMessageId.value === payload.messageId) {
    //         console.log('unlisten_finish_chunks called event.', event);
    //         now_messaging.value = payload.responseHtml;
    //         now_messaging_raw = payload.response;
    //         nextTick(() => {
    //             if (articleDom) {
    //                 articleDom.scrollTo(0, articleDom.scrollHeight);
    //             }
    //         });
    //     }
    // });
    // unlisten_timeout_stream = await listen('timeout_stream', (event) => {
    //     console.log('timeout_stream id:', event.payload);
    //     const messageId = event.payload;

    //     if (messageId === lastWaitingMessageId.value) {
    //         is_thinking.value = false;

    //         const lastAssistanceMessage = { 'role': 'assistant', 'content': now_messaging_raw, 'content_html': now_messaging.value };
    //         all_messages.value.push(lastAssistanceMessage);
    //         now_messaging.value = "";
    //         now_messaging_raw = "";
    //         lastWaitingMessageId.value = "";

    //         nextTick(() => {
    //             if (articleDom) {
    //                 articleDom.scrollTo(0, articleDom.scrollHeight);
    //             }
    //         });
    //     }
    // });
    // unlisten_finish_chunks = await listen('finish_chunks', (event) => {
    //     console.log('called, finish_chunks', event.payload);
    //     const payload = event.payload;

    //     if (lastWaitingMessageId.value === payload.messageId) {
    //         is_thinking.value = false;
    //         if (payload.response) {
    //             const lastAssistanceMessage = { 'role': 'assistant', 'content': payload.response, 'content_html': payload.responseHtml };
    //             all_messages.value.push(lastAssistanceMessage);
    //             now_messaging.value = "";
    //             now_messaging_raw = "";
    //         } else {
    //             const lastAssistanceMessage = { 'role': 'assistant', 'content': now_messaging_raw, 'content_html': now_messaging.value };
    //             all_messages.value.push(lastAssistanceMessage);
    //             now_messaging.value = "";
    //             now_messaging_raw = "";
    //             lastWaitingMessageId.value = "";
    //         }
    //         // nextTick(() => {
    //         //     if (articleDom) {
    //         //         articleDom.scrollTo(0, articleDom.scrollHeight);
    //         //     }
    //         // });
    //     }
    // });

    refleshAssistants();
});
const refleshAssistants = () => {

    invoke('reflesh_assistants').then(async res => {
        console.log('response.', res);
        titleList.value = JSON.parse(res);
        // titles.values = 
    });
};
const loadContent = (id) => {
    invoke('load_messages', { id }).then(async res => {
        console.log('load response.', res);
        console.log('data; ', JSON.parse(res));
        // const lastAssistanceMessage = { 'role': 'assistant', 'content': event.payload, 'content_html': now_messaging.value };
        all_messages.value = JSON.parse(res);
    });
}
const searchResultListSorted = computed(() => {
    return searchResultList.value.sort((a, b) => {
        return a.time === b.time ? 0 : a.time < b.time ? 1 : -1;
    });
});
const titleListSorted = computed(() => {
    return titleList.value.sort((a, b) => {
        return a.created === b.created ? 0 : a.created < b.created ? 1 : -1;
    });
});
//methods
const changeContent = (title) => {
    invoke('change_message', { id: title.id, name: title.name }).then(async res => {
        title.isEditing = false;
        refleshTitles();
    });
}

const deleteContent = (id) => {
    invoke('delete_message', { id }).then(async res => {
        console.log('delete response.', res);
        refleshTitles();
    });
}
const clear_assistant = () => {
    window.location.reload();
};
const save_assistant = () => {

    //save model and chat data.
    invoke('make_assistant', {
        params: JSON.stringify({
            message: message.value,
            assistant_name: assistant_name.value,
            instructions: instructions.value,
        })
    }).then(async res => {
        clear_search();
        refleshTitles();
        console.log('response.', res);
    });
};

const sendMessageStream = () => {
    console.log('sendMessageStream.');
    // const messageId = uuidv4();
    // lastWaitingMessageId.value = messageId;

    // const userMessage = { 'role': send_role.value, 'content': message.value };
    // all_messages.value.push(userMessage);
    // now_messaging.value = "";
    // message.value = '';

    invoke('assistents_test', {
        params: JSON.stringify({
            message: message.value,
            // model: ai_name.value,
            // temperature: 0.9,
            // max_tokens: 2048,
            // messageId: messageId,
        }),
        timeoutSec: timeoutSec.value,
    }).then(async res => {
        console.log('send_message_and_callback_stream response.', res);
    }).catch(err => {
        console.log('error:', err);
    });

    nextTick(() => {
        if (articleDom) {
            articleDom.scrollTo(0, articleDom.scrollHeight);
        }
    });
}

const clear_search = () => {
    errorMsg.value = '';
    search_word.value = '';
    searchResultList.value = [];
    all_messages.value = [];
}

const search = () => {
    errorMsg.value = '';
    if (!search_word.value || search_word.value.length < 2) {
        errorMsg.value = "please input search word 2 or more characters.";
        return;
    }
    invoke('search_assistants', {
        word: search_word.value,
    }).then(async res => {
        const json = JSON.parse(res);
        console.log('response.',);
        searchResultList.value = json;
    });
}

</script>

<template>
    <div class="container" style="dislpay: flex;">
        <Multipane class="vertical-panes w-full" layout="vertical">
            <div>
                <div style="width: 15rem;">
                    <input type="text" v-model="search_word" @keypress.enter="search" />
                    <div v-if="errorMsg" style="font-weight: bold; color: #CA2A2A;">{{ errorMsg }}</div>
                    <button @click="search">search</button>
                    <!-- <button @click="reflesh_index">reflesh index</button> -->
                    <button @click="clear_search">clear search</button>
                </div>
                <div v-if="searchResultListSorted && searchResultListSorted.length > 0"
                    style="overflow-y: scroll; max-height: 90vh;">
                    <div v-for="searchResult in searchResultListSorted" @click="loadContent(searchResult.id)"
                        :key="'search_result_id_' + searchResult.id"
                        style="max-width: 400px; font-weight: bold; color: #CA2A2A; cursor: pointer; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
                        {{ searchResult.title }}
                    </div>
                </div>
                <div v-else style="overflow-y: scroll; max-height: 90vh;">
                    <div v-for="title in titleListSorted" :key="'title_id_' + title.id"
                        style="display: flex; cursor: pointer; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
                        <template v-if="!title.isEditing">
                            <div style="flex: glow; max-width: 400px;" @click="loadContent(title.id)">{{ title.name ||
                                '(タイトルなし)' }}</div>
                            <div style="flex: 1">
                                <button @click="deleteContent(title.id)" class="button-sm">削</button>
                                <button @click="() => title.isEditing = true" class="button-sm">変</button>
                            </div>
                            <!--<div>
                            title all json:
                            {{ JSON.stringify(title) }}
                        </div> -->
                        </template>
                        <template v-else>
                            <div style="flex: glow; max-width: 400px;">
                                <input type="text" v-model="title.name" @blur="changeContent(title)"
                                    @keypress.enter="changeContent(title)" />
                            </div>
                        </template>
                    </div>
                </div>
            </div>
            <MultipaneResizer></MultipaneResizer>
            <div style="flex-direction: column; width:100%; flex: 1 1 0%; overflow: hidden;">
                <div style="display: flex; justify-content: space-between;">
                    <h3>Model:
                        <select style="font-size: 2rem;" v-model="ai_name">
                            <option v-for="value in AI_MODELS" :value="value" :key="'ai_name_' + value">
                                {{ value }}</option>
                        </select>
                    </h3>
                    <button @click="save_assistant">save</button>
                    <button @click="clear_assistant">clear</button>
                </div>

                <div>click "send" or ctrl + enter to make Assistant.</div>

                <div style="display: flex;">
                    <span>tempareture: </span>
                    <input type="text" v-model="tempareture" />
                    <span>timeout: </span>
                    <input type="text" v-model="timeoutSec" />
                </div>

                <div>
                    <input type="text" v-model="assistant_name" placeholder="ギャル魔王" />
                </div>
                <div class="w-full">
                    <input type="text" class="w-full" v-model="instructions"
                        placeholder="あなたはギャルな魔王です。世界を支配する魔王として、ギャル語で質問に答えてください" />
                </div>
                <textarea type="text" v-model="message" @keydown.ctrl.enter="sendMessageStream"
                    style="height: 3rem; width: 80%;"></textarea>
            </div>
            <div><button @click="sendMessageStream">send_assistant</button></div>

        </Multipane>
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
