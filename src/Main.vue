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
import { AI_MODELS, AUDIO_MODELS } from './constants';


const CHAT_TYPE_LIST = [
    { id: "chat", disp: "Chat" },
    { id: "assistant", disp: "Assistant" },
    { id: "audio", disp: "Audio" },
]

const LIST_MODE = {
    CONVERSATIONS: "conversations",
    THREAD: "thread",
    OPENAI_FILES: "openai_files",
    VECTORS: "openai_vectors",
}

const router = useRouter();

const message = ref("");
const listMode = ref(LIST_MODE.CONVERSATIONS)
const all_messages = ref([]);

const chatType = ref("chat");

const now_messaging = ref("");
const threadId = ref("");

let now_messaging_raw = "";
const is_thinking = ref(false);
const disp_raw_text_indexes = ref([]);
const send_role = ref("user");
const tempareture = ref(0.9);
const template = ref("");
const ai_name = ref("gpt-4o-mini");
const audio_model = ref("whisper-1");
const assistant_id = ref("");

const search_word = ref("");
const errorMsg = ref("");
const lastWaitingMessageId = ref("");
const timeoutSec = ref(180);

const conversationList = ref([]);
const threadList = ref([]);
const assistantList = ref([]);
const openAIFileList = ref([]);
const vectorList = ref([]);
const searchResultList = ref([]);

//audio
const fileInputAudio = ref(null);
const audioFile = ref(null);
//image
const imageUrl = ref("");
const fileInputImage = ref(null);
const fileInputImageChat = ref(null);
const imageFile = ref(null);
const imageFileChat = ref(null);

let articleDom = null;

let unlisten_stream_chunk = null;
let unlisten_finish_chunks = null;
let unlisten_stream_error = null;
let unlisten_stream_openai_error = null;
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
    if (unlisten_stream_openai_error) {
        unlisten_stream_openai_error();
    }
    if (unlisten_timeout_stream) {
        unlisten_timeout_stream();
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
    unlisten_stream_openai_error = await listen('stream_openai_error', (event) => {
        is_thinking.value = false;

        now_messaging.value = `<h3>OpenAIError</h3><p>${event}</p>`;
        nextTick(() => {
            if (articleDom) {
                articleDom.scrollTo(0, articleDom.scrollHeight);
            }
        });
    });
    unlisten_stream_chunk = await listen('stream_chunk', (event) => {
        console.log("unlisten_stream_chunk vent.")
        console.log('streamdata:', event.payload);
        const payload = event.payload;

        // is_thinking.value = false;
        if (lastWaitingMessageId.value === payload.messageId) {
            console.log('unlisten_finish_chunks called event.', event);
            now_messaging.value = payload.responseHtml;
            now_messaging_raw = payload.response;
            if (payload.threadId) {
                threadId.value = payload.threadId;
            }

            nextTick(() => {
                if (articleDom) {
                    articleDom.scrollTo(0, articleDom.scrollHeight);
                }
            });
        }
    });
    unlisten_timeout_stream = await listen('timeout_stream', (event) => {
        console.log('timeout_stream id:', event.payload);
        const messageId = event.payload;

        if (messageId === lastWaitingMessageId.value) {
            is_thinking.value = false;

            const lastAssistanceMessage = { 'role': 'assistant', 'content': now_messaging_raw, 'content_html': now_messaging.value };
            all_messages.value.push(lastAssistanceMessage);
            now_messaging.value = "";
            now_messaging_raw = "";
            lastWaitingMessageId.value = "";

            nextTick(() => {
                if (articleDom) {
                    articleDom.scrollTo(0, articleDom.scrollHeight);
                }
            });
        }
    });
    unlisten_finish_chunks = await listen('finish_chunks', (event) => {
        console.log('called, finish_chunks', event.payload);
        const payload = event.payload;

        if (lastWaitingMessageId.value === payload.messageId) {
            is_thinking.value = false;
            if (payload.response) {
                const lastAssistanceMessage = { 'role': 'assistant', 'content': payload.response, 'content_html': payload.responseHtml };
                all_messages.value.push(lastAssistanceMessage);
                now_messaging.value = "";
                now_messaging_raw = "";
            } else {
                const lastAssistanceMessage = { 'role': 'assistant', 'content': now_messaging_raw, 'content_html': now_messaging.value };
                all_messages.value.push(lastAssistanceMessage);
                now_messaging.value = "";
                now_messaging_raw = "";
                lastWaitingMessageId.value = "";
            }
            if (payload.threadId) {
                threadId.value = payload.threadId;
            }
            // nextTick(() => {
            //     if (articleDom) {
            //         articleDom.scrollTo(0, articleDom.scrollHeight);
            //     }
            // });
        }
    });
    refleshAssistants();
    refleshConversations();
});
const refleshConversations = () => {

    invoke('reflesh_titles').then(async res => {
        console.log('response.', res);
        conversationList.value = JSON.parse(res);
        // titles.values = 
    });

    invoke('reflesh_threads').then(async res => {
        console.log('reflesh_threads response.', res);
        threadList.value = JSON.parse(res);
        // titles.values = 
    });

    invoke('reflesh_openai_files').then(async res => {
        console.log('reflesh_openai_files response.', res);
        openAIFileList.value = JSON.parse(res);
        // titles.values =
    });

    invoke('reflesh_vectors').then(async res => {
        console.log('reflesh_vector response.', res);
        vectorList.value = JSON.parse(res);
        // titles.values =
    });
};
const isThread = (id) => {
    return id.startsWith("thread_");
}
// const getThreadAndAssistantId = (id) => {
//     // thread_xxxxxxassistant_yyyyyy -> (xxxxxx, yyyyyy)
//     console.log('getThreadAndAssistantId:', id);
//     if (id.startsWith("thread_")) {
//         const parts = id.split("thread_");
//         const tmpList = parts[1].split("asst_");
//         const threadId = tmpList[0];
//         const assistantId = tmpList[1];
//         return ["thread_" + threadId, "asst_" + assistantId];
//     } else {
//         return ["", ""]; // 空の文字列の配列を返す
//     }
// }
const loadThread = (id) => {
    console.log('loadThread called.', id);
}
const loadContent = (id) => {
    invoke('load_messages', { id }).then(async res => {
        console.log('load response.', res);
        console.log('data; ', JSON.parse(res));
        // const lastAssistanceMessage = { 'role': 'assistant', 'content': event.payload, 'content_html': now_messaging.value };
        const conversation = JSON.parse(res);
        all_messages.value = conversation.messages;

        if (isThread(id)) {
            threadId.value = id;
            if (conversation.assistant_id) {
                console.log('assistantId is!.', conversation.assistant_id);
                assistant_id.value = conversation.assistant_id;
                chatType.value = "assistant";
            } else {
                console.log('assistantId is empty.');
            }
        }
    });
};

const vectorListSorted = computed(() => {
    return vectorList.value.sort((a, b) => {
        return a.time === b.time ? 0 : a.time < b.time ? 1 : -1;
    });
});
const openAIFileListSorted = computed(() => {
    return openAIFileList.value.sort((a, b) => {
        return a.time === b.time ? 0 : a.time < b.time ? 1 : -1;
    });
});
const threadListSorted = computed(() => {
    return threadList.value.sort((a, b) => {
        return a.time === b.time ? 0 : a.time < b.time ? 1 : -1;
    });
});
const searchResultListSorted = computed(() => {
    return searchResultList.value.sort((a, b) => {
        return a.time === b.time ? 0 : a.time < b.time ? 1 : -1;
    });
});
const conversationListSorted = computed(() => {
    return conversationList.value.sort((a, b) => {
        return a.time === b.time ? 0 : a.time < b.time ? 1 : -1;
    });
});
//methods
//image
const imageFilePick = async () => {
    clearSelectedFile();
    console.log('fileInput.value:', fileInputImage.value);
    fileInputImage.value.click();
};
const imageFilePicked = async (event) => {
    const files = event.target.files;
    console.log('files:', files);
    console.log('fileInputImage.value:', fileInputImage.value);
    // readFile(files[0], true);
    imageFile.value = files[0]
};
const imageFileChatPick = async () => {
    clearSelectedFile();
    console.log('fileInput.value:', fileInputImageChat.value);
    fileInputImageChat.value.click();
};
const imageFileChatPicked = async (event) => {
    const files = event.target.files;
    console.log('files:', files);
    console.log('fileInputImage.value:', fileInputImageChat.value);
    // readFile(files[0], true);
    imageFileChat.value = files[0]
};
const readImageFileChat = async () => {
    if (imageFileChat.value) {
        return new Promise((resolve, reject) => {
            const fileReader = new FileReader();
            const fileName = imageFileChat.value.name;

            fileReader.onload = () => {
                const fileBody = fileReader.result;
                resolve({ fileBody, fileName });
            };
            fileReader.onerror = (error) => {
                reject(error);
            };
            fileReader.readAsDataURL(imageFileChat.value);
        });
    } else {
        return null;
    }
};
const readImageFile = async () => {
    if (imageFile.value) {
        return new Promise((resolve, reject) => {
            const fileReader = new FileReader();
            const fileName = imageFile.value.name;

            fileReader.onload = () => {
                const fileBody = fileReader.result;
                resolve({ fileBody, fileName });
            };
            fileReader.onerror = (error) => {
                reject(error);
            };
            fileReader.readAsDataURL(imageFile.value);
        });
    } else {
        return null;
    }
};
//audio
const audioFilePick = async () => {
    clearSelectedFile();
    console.log('fileInput.value:', fileInputAudio.value);
    fileInputAudio.value.click();
};
const audioFilePicked = async (event) => {
    const files = event.target.files;
    console.log('files:', files);
    console.log('fileInputAudio.value:', fileInputAudio.value);
    // readFile(files[0], true);
    audioFile.value = files[0]
};
const audioTranscribe = async () => {
    //TODO メモリを有効に使うために、本当はここでバイナリ読み込むのではなく、Rust側で処理を行いたい（ブラウザのメモリ使用量を抑えるため）
    //ファイルを読み込んで、invokeする
    if (audioFile.value) {
        const fileReader = new FileReader();
        const fileName = audioFile.value.name;
        fileReader.addEventListener('load', () => {
            const fileBody = fileReader.result;
            invoke('audio_transcribe', { filebody: fileBody, filename: fileName }).then(async res => {
                console.log('res:', res);
                const response = JSON.parse(res);
                console.log('response:', response);

                const lastAssistanceMessage = { 'role': 'assistant', 'content': response.text, 'content_html': response.text };
                all_messages.value.push(lastAssistanceMessage);
                now_messaging.value = "";
                now_messaging_raw = "";
                lastWaitingMessageId.value = "";
                clearSelectedFile();
            }).catch(err => {
                console.error('error:', err);
                now_messaging.value = `<pre>${err}</pre>`;
            });
        });
        fileReader.readAsDataURL(audioFile.value)
    }
};
const clearSelectedFile = () => {
    // fileInputAudio.value = '';
    audioFile.value = null;
    imageFile.value = null;
    imageFileChat.value = null;
};
const changeContent = (conversation) => {
    invoke('change_message', { id: conversation.id, title: conversation.title }).then(async res => {
        conversation.isEditing = false;
        refleshConversations();
    });
}
const deleteVector = (id) => {
    console.log('delete_vector');
    invoke('delete_vector', { id }).then(async res => {
        console.log('delete vector response.', res);
        refleshConversations();
    });
}
const deleteOpenAIFile = (id) => {
    console.log('delete_openai_file');
    invoke('delete_openai_file', { id }).then(async res => {
        console.log('delete openai file response.', res);
        refleshConversations();
    });
}
const deleteThread = (id) => {
    console.log('delete_thread');
    invoke('delete_thread', { id }).then(async res => {
        console.log('delete thread response.', res);
        refleshConversations();
    });
}

const deleteContent = (id) => {
    invoke('delete_message', { id }).then(async res => {
        console.log('delete response.', res);
        refleshConversations();
    });
}
const add_template = () => {
    message.value += "\n" + template.value;
};
const new_chat = () => {
    window.location.reload();
};
const save_chat = (e, saveThread = false) => {

    //save model and chat data.
    console.log('saveThread: ', saveThread);
    invoke('save_chat', {
        params: JSON.stringify({
            data: all_messages.value.map(x => ({
                role: x.role,
                content: x.content,
            })),
            thread_id: threadId.value,
            assistant_id: assistant_id.value,
            save_thread: saveThread,
        })
    }).then(async res => {
        clear_search();
        refleshConversations();
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
const refleshAssistants = () => {

    invoke('reflesh_assistants').then(async res => {
        console.log('response.', res);
        assistantList.value = JSON.parse(res);
        // titles.values = 
    });
};

const translateToJp = () => {
    message.value = "translate to japanese below.\n" + message.value;
    sendMessageStream();
}
const translateToEn = () => {
    message.value = "translate to English below.\n" + message.value;
    sendMessageStream();
}
const sendMessageStream = async () => {
    console.log('sendMessageStream chat called.');
    const messageId = uuidv4();
    lastWaitingMessageId.value = messageId;

    if (chatType.value === "chat") {
        threadId.value = '';
        console.log('sendMessageStream chat called.');
        const userMessage = { 'role': send_role.value, 'content': message.value };
        all_messages.value.push(userMessage);

        imageFile.value = "";
        now_messaging.value = "";
        message.value = '';


        // invoke('send_message_and_callback_stream', {
        const data = {
            messages: all_messages.value,
            model: ai_name.value,
            temperature: 0.9,
            max_tokens: 2048,
            messageId: messageId,
            imageUrl: imageUrl.value,
        };
        //画像がアップされてたら取得する
        if (imageFileChat.value) {
            const result = await readImageFileChat();
            console.log('result:', result);
            data["filename"] = result.fileName;
            data["filebody"] = result.fileBody;
        }

        imageFile.value = "";
        imageFileChat.value = "";
        now_messaging.value = "";
        message.value = '';
        invoke('start_chat', {
            params: JSON.stringify(data),
            timeoutSec: timeoutSec.value,
        }).then(async res => {
            console.log('send_message_and_callback_stream response.', res);
        });
    } else if (chatType.value === "assistant") {
        console.log('sendMessageStream assystant called.');
        const messageId = uuidv4();
        lastWaitingMessageId.value = messageId;

        const userMessage = { 'role': send_role.value, 'content': message.value };
        all_messages.value.push(userMessage);

        const data = {
            messages: all_messages.value.slice(-1),
            assistant_id: assistant_id.value,
            messageId: messageId,
            threadId: threadId.value,
            imageUrl: imageUrl.value,
        };
        //画像がアップされてたら取得する
        if (imageFile.value) {
            const result = await readImageFile();
            data["filename"] = result.fileName;
            data["filebody"] = result.fileBody;
        }

        imageFile.value = "";
        now_messaging.value = "";
        message.value = '';

        invoke('make_new_thread', {
            params: JSON.stringify(data),
            // timeoutSec: timeoutSec.value,
        }).then(async res => {
            console.log('send_message_and_callback_stream response.', res);
        });
    } else {
        console.log(chatType.value, ' called.');
    }


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
    threadId.value = '';
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


</script>

<template>
    <div class="container" style="dislpay: flex;">
        <Multipane class="vertical-panes w-full" layout="vertical">
            <div>
                <div>
                    <label v-for="(value, key) in LIST_MODE" :key="'list_mode_' + key"><input type="radio"
                            v-model="listMode" :value="value" name="list_mode" />{{ value }}</label>
                </div>
                <div v-if="listMode === LIST_MODE.THREAD" style="overflow-y: scroll; max-height: 90vh;">
                    <div style="overflow-y: scroll; max-height: 90vh;">
                        <div v-for="thread in threadListSorted" :key="'thread_id_' + thread.id"
                            style="display: flex; cursor: pointer; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">

                            <div style="flex: glow; max-width: 400px;" @click="loadThread(thread.id)">
                                <img src="./assets/chatgpt.png" style="width: 20px; height:20px;" />Th:
                                {{ thread.name || thread.id }}
                            </div>
                            <div style="flex: 1">
                                <button @click="deleteThread(thread.id)" class="button-sm">削</button>
                            </div>
                        </div>
                    </div>
                </div>
                <div v-else-if="listMode === LIST_MODE.VECTORS" style="overflow-y: scroll; max-height: 90vh;">
                    <div style="overflow-y: scroll; max-height: 90vh;">
                        <div v-for="vector in vectorListSorted" :key="'vector_id_' + vector.id"
                            style="display: flex; cursor: pointer; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">

                            <div style="flex: glow; max-width: 400px;">
                                <img src="./assets/chatgpt.png" style="width: 20px; height:20px;" />Th:
                                {{ vector.name || vector.id }}
                            </div>
                            <div style="flex: 1">
                                <button @click="deleteVector(vector.id)" class="button-sm">削</button>
                            </div>
                        </div>
                    </div>
                </div>
                <div v-else-if="listMode === LIST_MODE.OPENAI_FILES" style="overflow-y: scroll; max-height: 90vh;">
                    <div style="overflow-y: scroll; max-height: 90vh;">
                        <div v-for="openAIFile in openAIFileListSorted" :key="'openai_file_id_' + openAIFile.id"
                            style="display: flex; cursor: pointer; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">

                            <div style="flex: glow; max-width: 400px;">
                                <img src="./assets/chatgpt.png" style="width: 20px; height:20px;" />Th:
                                {{ openAIFile.filename || openAIFile.id }}
                            </div>
                            <div style="flex: 1">
                                <button @click="deleteOpenAIFile(openAIFile.id)" class="button-sm">削</button>
                            </div>
                        </div>
                    </div>
                </div>
                <div v-else-if="listMode === LIST_MODE.CONVERSATIONS" style="overflow-y: scroll; max-height: 90vh;">
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
                        <div v-for="conversation in conversationListSorted" :key="'conversation_id_' + conversation.id"
                            style="display: flex; cursor: pointer; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
                            <template v-if="!conversation.isEditing">
                                <div style="flex: glow; max-width: 400px;" @click="loadContent(conversation.id)">
                                    <span v-if="conversation.id.startsWith('thread_')"><img src="./assets/chatgpt.png"
                                            style="width: 20px; height:20px;" />Th:</span>
                                    {{ conversation.title || '(タイトルなし)' }}
                                </div>
                                <div style="flex: 1">
                                    <button @click="deleteContent(conversation.id)" class="button-sm">削</button>
                                    <button @click="() => conversation.isEditing = true" class="button-sm">変</button>
                                </div>
                            </template>
                            <template v-else>
                                <div style="flex: glow; max-width: 400px;">
                                    <input type="text" v-model="conversation.title" @blur="changeContent(conversation)"
                                        @keypress.enter="changeContent(conversation)" />
                                </div>
                            </template>
                        </div>
                    </div>
                </div>
            </div>
            <MultipaneResizer></MultipaneResizer>
            <div style="flex-direction: column; width:100%; flex: 1 1 0%; overflow: scroll;">
                <div>
                    <label v-for="chatTypeObj in CHAT_TYPE_LIST" :key="'chat_type_' + chatTypeObj.id"><input
                            type="radio" v-model="chatType" :value="chatTypeObj.id" />{{ chatTypeObj.disp }}</label>
                </div>
                <div>chatType: {{ chatType }}</div>
                <div> <button @click="save_chat">save</button>
                    <button v-if="threadId" @click="save_chat($event, true)">save thread</button>
                    <button @click="new_chat">new chat</button>
                </div>
                <div v-if="chatType === 'chat'">
                    <div style="display: flex; justify-content: space-between;">
                        <h3>Model:
                            <select style="font-size: 2rem;" v-model="ai_name">
                                <option v-for="value in AI_MODELS" :value="value" :key="'ai_name_' + value">
                                    {{ value }}</option>
                            </select>
                        </h3>
                    </div>

                    <div>click "send" or ctrl + enter to send message.<label v-for="role in ROLES"
                            :key="'role_' + role">
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
                    <div>
                        <div>画像URL:<input type="text" style="width: 100%;" v-model="imageUrl" /></div>
                        <button @click="imageFileChatPick" style="padding: 5 px; margin-left: 5px;">画像ファイルUP</button>
                        <input type="file" style="display: none" ref="fileInputImageChat"
                            @change="imageFileChatPicked" />
                        <div v-if="imageFileChat">{{ imageFileChat.name }}</div>

                    </div>
                </div>
                <div v-else-if="chatType === 'assistant'">
                    <div style="display: flex; justify-content: space-between;">
                        <h3>Assistant:
                            <select style="font-size: 2rem;" v-model="assistant_id">
                                <option v-for="assistant in assistantList" :value="assistant.id"
                                    :key="'assistant_id_' + assistant.id">
                                    {{ assistant.name }}</option>
                            </select>
                        </h3>
                        <div>thread: {{ threadId }}</div>
                        <div>画像URL:<input type="text" style="width: 100%;" v-model="imageUrl" /></div>
                        <button @click="imageFilePick" style="padding: 5 px; margin-left: 5px;">画像ファイルUP</button>
                        <input type="file" style="display: none" ref="fileInputImage" @change="imageFilePicked" />
                        <div v-if="imageFile">{{ imageFile.name }}</div>
                    </div>
                </div>
                <div v-else-if="chatType === 'audio'">
                    <h3>Model:
                        <select style="font-size: 2rem;" v-model="audio_model">
                            <option v-for="value in AUDIO_MODELS" :value="value" :key="'whisper_name_' + value">
                                {{ value }}</option>
                        </select>
                    </h3>
                    <button @click="audioFilePick" style="padding: 5 px; margin-left: 5px;">オーディオファイル読込</button>
                    <input type="file" style="display: none" ref="fileInputAudio" @change="audioFilePicked" />
                    <button @click="audioTranscribe">Audio Transcribe</button>
                    <div v-if="audioFile">{{ audioFile.name }}</div>
                </div>
                <div style="display: flex; align-items: flex-end;">
                    <textarea type="text" v-model="message" @keydown.ctrl.enter="sendMessageStream"
                        style="height: 3rem; width: 80%;"></textarea>
                    <!-- <button @click="sendMessage">send</button> -->
                    <button @click="sendMessageStream">send！</button>
                    <!-- <button @click="translateToJp">translate to Jp</button>
                <button @click="translateToEn">translate to En</button> -->
                </div>

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
