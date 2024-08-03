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

const router = useRouter();

const message = ref("");
const all_messages = ref([]);

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
    // articleDom = document.getElementById('article');
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

    // refleshTitles();
});
const refleshTitles = () => {

    invoke('reflesh_titles').then(async res => {
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
        return a.time === b.time ? 0 : a.time < b.time ? 1 : -1;
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
const add_template = () => {
    message.value += "\n" + template.value;
};
const new_chat = () => {
    window.location.reload();
};
const save_chat = () => {

    //save model and chat data.
    invoke('save_chat', {
        params: JSON.stringify({
            data: all_messages.value.map(x => ({ role: x.role, content: x.content })),
        })
    }).then(async res => {
        clear_search();
        refleshTitles();
        console.log('response.', res);
    });

    // 'role': 'assistant', 'content': event.payload, 'content_html': now_messaging.value
};
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

const sendMessageStream10 = () => {
    invoke('embedding_test', {
        params: JSON.stringify({
            message: message.value,
        }),
        timeoutSec: timeoutSec.value,
    }).then(async res => {
        console.log('assistants_audio_translate_test response.', res);
    }).catch(err => {
        console.log('error:', err);
    });
};
const sendMessageStream9 = () => {
    invoke('assistants_tool_calls_test', {
        params: JSON.stringify({
            message: message.value,
        }),
        timeoutSec: timeoutSec.value,
    }).then(async res => {
        console.log('assistants_audio_translate_test response.', res);
    }).catch(err => {
        console.log('error:', err);
    });
};
const sendMessageStream8 = () => {
    invoke('assistants_vision_chat_test', {
        params: JSON.stringify({
            message: message.value,
        }),
        timeoutSec: timeoutSec.value,
    }).then(async res => {
        console.log('assistants_audio_translate_test response.', res);
    }).catch(err => {
        console.log('error:', err);
    });
};
const sendMessageStream7 = () => {
    console.log('sendMessageStream6.');

    invoke('assistants_audio_speech_test', {
        params: JSON.stringify({
            message: message.value,
        }),
        timeoutSec: timeoutSec.value,
    }).then(async res => {
        console.log('assistants_audio_translate_test response.', res);
    }).catch(err => {
        console.log('error:', err);
    });
};
const sendMessageStream6 = () => {
    console.log('sendMessageStream6.');

    invoke('assistants_audio_translate_test', {
        params: JSON.stringify({
            message: message.value,
        }),
        timeoutSec: timeoutSec.value,
    }).then(async res => {
        console.log('assistants_audio_translate_test response.', res);
    }).catch(err => {
        console.log('error:', err);
    });
};
const sendMessageStream5 = () => {
    console.log('sendMessageStream5.');

    invoke('assistants_audio_transcribe_test', {
        params: JSON.stringify({
            message: message.value,
        }),
        timeoutSec: timeoutSec.value,
    }).then(async res => {
        console.log('send_message_and_callback_stream response.', res);
    }).catch(err => {
        console.log('error:', err);
    });
}
const sendMessageStream4 = () => {
    console.log('sendMessageStream4.');

    invoke('assistents_code_interpreter_test', {
        params: JSON.stringify({
            message: message.value,
        }),
        timeoutSec: timeoutSec.value,
    }).then(async res => {
        console.log('send_message_and_callback_stream response.', res);
    }).catch(err => {
        console.log('error:', err);
    });
}
const sendMessageStream3 = () => {
    console.log('sendMessageStream3.');

    invoke('assistents_file_search_test', {
        params: JSON.stringify({
            message: message.value,
        }),
        timeoutSec: timeoutSec.value,
    }).then(async res => {
        console.log('send_message_and_callback_stream response.', res);
    }).catch(err => {
        console.log('error:', err);
    });
}
const sendMessageStream2 = () => {
    console.log('sendMessageStream2.');

    invoke('assistents_stream_test', {
        params: JSON.stringify({
            message: message.value,
        }),
        timeoutSec: timeoutSec.value,
    }).then(async res => {
        console.log('send_message_and_callback_stream response.', res);
    }).catch(err => {
        console.log('error:', err);
    });

}
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


    // invoke('send_message_and_callback_stream', {
    //     params: JSON.stringify({
    //         messages: all_messages.value,
    //         model: ai_name.value,
    //         temperature: 0.9,
    //         max_tokens: 2048,
    //         messageId: messageId,
    //     }),
    //     timeoutSec: timeoutSec.value,
    // }).then(async res => {
    //     console.log('send_message_and_callback_stream response.', res);
    // });

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
const reflesh_index = () => {
    invoke('reflesh_index').then(async res => {
        console.log('response.', res);
    });
}
const search = () => {
    errorMsg.value = '';
    if (!search_word.value || search_word.value.length < 2) {
        errorMsg.value = "please input search word 2 or more characters.";
        return;
    }
    invoke('search_conversations', {
        word: search_word.value,
    }).then(async res => {
        const json = JSON.parse(res);
        console.log('response.',);
        searchResultList.value = json;
    });
}
const cancel = () => {
    const lastAssistanceMessage = { 'role': 'assistant', 'content': now_messaging_raw, 'content_html': now_messaging.value };
    all_messages.value.push(lastAssistanceMessage);
    lastWaitingMessageId.value = '';
    now_messaging.value = "";
    now_messaging_raw = "";
}
const goOn = () => {
    const lastAssistanceMessage = { 'role': 'assistant', 'content': now_messaging_raw, 'content_html': now_messaging.value };
    all_messages.value.push(lastAssistanceMessage);
    lastWaitingMessageId.value = '';
    now_messaging.value = "";
    now_messaging_raw = "";
    message.value = 'go on';
    sendMessageStream();
}

const ROLES = ['user', 'system'];
const TEMPLATES = [
    `If the question cannot be answered using the information provided answer with "I don't know"`,
    "Let's think logically, step by step. ",
    "First,", "Let's think about this logically.",
    "Let's solve this problem by splitting it into steps."];

const AI_MODELS = [/*'gpt-4-32k',*/ "gpt-4o-mini", "gpt-4", "gpt-4o", "gpt-4-turbo"];
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
                        style="max-width: 400px; font-weight: bold; color: #CA2A2A; #ewe; cursor: pointer; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
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
                    <button @click="save_chat">save</button>
                    <button @click="new_chat">new chat</button>
                </div>

                <div>click "send" or ctrl + enter to send message.<label v-for="role in ROLES" :key="'role_' + role">
                        <input type="radio" v-model="send_role" :value="role" />{{ role }}
                    </label></div>
                <div>
                    <div style="display: flex;">
                        <span>tempareture: </span>
                        <input type="text" v-model="tempareture" />
                        <span>timeout: </span>
                        <input type="text" v-model="timeoutSec" />
                    </div>
                    <div><button @click="add_template">add template</button>
                        <select v-model="template">
                            <option v-for="value in TEMPLATES" :value="value" :key="'template_' + value">{{ value }}
                            </option>
                        </select>
                    </div>
                </div>
                <div style="display: flex; align-items: flex-end;">
                    <textarea type="text" v-model="message" @keydown.ctrl.enter="sendMessageStream"
                        style="height: 3rem; width: 80%;"></textarea>
                    <!-- <button @click="sendMessage">send</button> -->
                    <!-- <button @click="translateToJp">translate to Jp</button>
                <button @click="translateToEn">translate to En</button> -->
                </div>
                <div><button @click="sendMessageStream">send_assistant</button></div>
                <div><button @click="sendMessageStream2">send_assistant_stream</button></div>
                <div><button @click="sendMessageStream3">FileSearch</button></div>
                <div><button @click="sendMessageStream4">Code Interpritter</button></div>
                <div><button @click="sendMessageStream5">audio_transcribe</button></div>
                <div><button @click="sendMessageStream6">audio translate</button></div>
                <div><button @click="sendMessageStream7">audio_speech</button></div>
                <div><button @click="sendMessageStream8">vision chat</button></div>
                <div><button @click="sendMessageStream9">tool calls</button></div>
                <div><button @click="sendMessageStream10">embedding</button></div>

                <div id="article" class="markdown"
                    style="overflow-y: scroll; max-height: 70vh; word-break: break-all; ">
                    <article v-for="(msg, ind) in all_messages" :key="'msg_' + ind"
                        :style="ind > 0 ? 'margin-top: 2rem;' : ''">
                        <div v-if="msg.role == 'user' || msg.role == 'system'">
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
                            <p v-if="disp_raw_text_indexes.includes(ind)" style="white-space:pre-wrap;">{{ msg.content
                                }}</p>
                            <div v-else v-html="msg.content_html || msg.content"></div>
                            <button v-if="msg.content_html.replace('<p>', '').replace('</p>', '') != msg.content"
                                @click="toggleDisplay(ind)">
                                <span v-if="disp_raw_text_indexes.includes(ind)">display formatted text</span><span
                                    v-else>display
                                    raw text</span>
                            </button>
                        </div>
                    </article>
                    <article v-show="is_thinking || now_messaging" style="margin-top: 2rem;">
                        <div><span>chatGPT</span></div>
                        <div v-if="now_messaging" v-html="now_messaging"></div>
                        <p v-else>I'm thinking...</p>
                    </article>
                    <div> for debug: now messageId: {{ lastWaitingMessageId }}</div>
                    <article v-if="all_messages.length > 0">
                        <button @click="goOn">go on</button><button @click="cancel">cancel</button>
                    </article>
                </div>
            </div>
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
