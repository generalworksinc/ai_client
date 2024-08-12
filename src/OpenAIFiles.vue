<script setup>
// This starter template is using Vue 3 <script setup> SFCs
// Check out https://vuejs.org/api/sfc-script-setup.html#script-setup
import { invoke, convertFileSrc } from '@tauri-apps/api/tauri'
import { open } from '@tauri-apps/api/dialog';
import { emit, listen } from '@tauri-apps/api/event';
import { useRouter } from 'vue-router';
import { ref, nextTick, onMounted, onUnmounted, computed } from "vue";
import { Multipane, MultipaneResizer } from './lib/multipane';
import { v4 as uuidv4 } from 'uuid';


const LIST_MODE = {
    OPENAI_FILES: "openai_files",
    VECTORS: "openai_vectors",
}

const router = useRouter();

const message = ref("");
const vectorName = ref("");
const listMode = ref(LIST_MODE.OPENAI_FILES)

const file_name = ref("");
const file_detail = ref("");

const search_word = ref("");
const errorMsg = ref("");

const openAIFileList = ref([]);
const vectorList = ref([]);

const selectedAIFileListForVector = ref([]);

const searchResultList = ref([]);

const fileDataList = ref(new Set());


const openAIFilePick = async () => {
    const filePathList = await open({
        multiple: true,
        // filters: [{
        //     name: 'Image',
        //     extensions: ['png', 'jpeg']
        // }]
    });
    for (const filePath of filePathList) {
        fileDataList.value.add(filePath);
    }
    console.log('fileDataList:', fileDataList);
};

const removeFile = (file) => {
    fileDataList.value.delete(file);
};
const deleteVector = (id) => {
    console.log('delete_vector');
    invoke('delete_vector', { id }).then(async res => {
        console.log('delete vector response.', res);
        refleshFiles();
    });
}
const deleteOpenAIFile = (id) => {
    console.log('delete_openai_file');
    invoke('delete_openai_file', { id }).then(async res => {
        console.log('delete openai file response.', res);
        refleshFiles();
    });
}
const readFile = async () => {
    return Promise.all(fileDataList.value.map(async (fileAssistant) => {
        return new Promise((resolve, reject) => {
            const fileReader = new FileReader();
            const fileName = fileAssistant.name;

            fileReader.onload = () => {
                const fileBody = fileReader.result;
                resolve([fileBody, fileName]);
            };
            fileReader.onerror = (error) => {
                reject(error);
            };
            fileReader.readAsDataURL(fileAssistant);
        });
    }));

};

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
    refleshFiles();
});
const refleshFiles = () => {

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
const loadContent = (id) => {
    invoke('load_messages', { id }).then(async res => {
        console.log('load response.', res);
        console.log('data; ', JSON.parse(res));
        // const lastAssistanceMessage = { 'role': 'assistant', 'content': event.payload, 'content_html': now_messaging.value };
        // all_messages.value = JSON.parse(res);
    });
}

// computed
const searchResultListSorted = computed(() => {
    return searchResultList.value.sort((a, b) => {
        return a.time === b.time ? 0 : a.time < b.time ? 1 : -1;
    });
});
const openAIFileListSorted = computed(() => {
    return openAIFileList.value.sort((a, b) => {
        return a.created === b.created ? 0 : a.created < b.created ? 1 : -1;
    });
});
const vectorListSorted = computed(() => {
    return vectorList.value.sort((a, b) => {
        return a.created === b.created ? 0 : a.created < b.created ? 1 : -1;
    });
});

//methods
const filePathRegex = /[^\\/]+$/;
const getFileName = (filePath) => {
    // 正規表現を使用してファイル名を取得
    const match = filePath.match(filePathRegex);
    return match ? match[0] : "(no file)"; // マッチがあればファイル名を返す
}

const unselectOpenAIFile = (file) => {
    const fileIndex = selectedAIFileListForVector.value.indexOf(file);
    if (fileIndex >= 0) {
        selectedAIFileListForVector.value.splice(fileIndex, 1);
    }
};

const selectOpenAIFile = (file) => {
    if (selectedAIFileListForVector.value.indexOf(file) < 0) {
        selectedAIFileListForVector.value.push(file)
    }
};
const changeContent = (title) => {
    invoke('change_message', { id: title.id, name: title.name }).then(async res => {
        title.isEditing = false;
        refleshFiles();
    });
}

const deleteAssistant = (id) => {
    invoke('delete_assistant', { id }).then(async res => {
        console.log('delete_assistant.', res);
        refleshFiles();
    });
}
const relaod_page = () => {
    window.location.reload();
};
const save_file = async () => {
    console.log('save_file', listMode.value, selectedAIFileListForVector.value.length);
    if (listMode.value === LIST_MODE.VECTORS) {
        // vectorを生成する
        if (selectedAIFileListForVector.value.length > 0) {
            invoke('make_vector', {
                params: JSON.stringify({
                    vector_name: vectorName.value,
                    open_ai_file_id_list: selectedAIFileListForVector.value.map((f) => f.id),
                })
            }).then(async res => {
                clear_search();
                refleshFiles();
                console.log('response.', res);
            });
        }
    } else if (listMode.value === LIST_MODE.OPENAI_FILES) {
        //画像がアップされてたら取得する
        let fileList = [];
        if (fileDataList.value.length > 0) {
            const assistFileList = await readFile();
            for (const result of assistFileList) {
                console.log('result.fileName: ', result.fileName);
            }
            fileList = assistFileList;
            // data["filename"] = result.fileName;
            // data["filebody"] = result.fileBody;
        }

        //save model and chat data.
        invoke('upload_files', {
            params: JSON.stringify({
                message: message.value,
                file_list: [...fileDataList.value],
            })
        }).then(async res => {
            clear_search();
            refleshFiles();
            console.log('response.', res);
        });
    }

};

const sendMessageStream = () => {
    console.log('sendMessageStream.');

    invoke('assistents_test', {
        params: JSON.stringify({
            message: message.value,
        }),
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
    fileDataList.value.clear();
    // all_messages.value = [];
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
                <div>
                    <label v-for="(value, key) in LIST_MODE" :key="'list_mode_' + key"><input type="radio"
                            v-model="listMode" :value="value" name="list_mode" />{{ value }}</label>
                </div>
                <div v-if="listMode === LIST_MODE.VECTORS" style="overflow-y: scroll; max-height: 90vh;">
                    <div style="overflow-y: scroll; max-height: 90vh;">
                        <div style="overflow-y: scroll; max-height: 90vh;">
                            <div v-for="vector in vectorListSorted" :key="'openai_file_id_' + vector.id"
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
                </div>
                <div v-if="listMode === LIST_MODE.OPENAI_FILES">
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
            </div>
            <MultipaneResizer></MultipaneResizer>
            <div style="flex-direction: column; width:100%; flex: 1 1 0%; overflow: scroll;">
                <div v-if="listMode === LIST_MODE.VECTORS">
                    <div>Vector生成に利用する画像</div>
                    <div v-for="openAIFile in selectedAIFileListForVector"
                        :key="'selected_openai_file_id_' + openAIFile.id"
                        style="display: flex; cursor: pointer; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">

                        <div style="flex: glow; max-width: 800px; overflow:hidden;">
                            <img src="./assets/chatgpt.png" style="width: 20px; height:20px;" />Th:
                            {{ openAIFile.filename || openAIFile.id }}
                        </div>
                        <div style="flex: 1">
                            <button @click="unselectOpenAIFile(openAIFile)" class="button-sm">除外</button>
                        </div>
                    </div>
                    <div class="mt-5">全てのアップ済み画像</div>
                    <div v-for="openAIFile in openAIFileListSorted.filter((f) => selectedAIFileListForVector.indexOf(f) < 0)"
                        :key="'openai_file_id_' + openAIFile.id"
                        style="display: flex; cursor: pointer; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">

                        <div style="flex: glow; max-width: 800px; overflow:hidden;">
                            <img src="./assets/chatgpt.png" style="width: 20px; height:20px;" />Th:
                            {{ openAIFile.filename || openAIFile.id }}
                        </div>
                        <div style="flex: 1">
                            <button @click="selectOpenAIFile(openAIFile)" class="button-sm">選択</button>
                        </div>
                    </div>
                    <div>
                        <input type="text" v-model="vectorName" placeholder="Financial Statements" />
                    </div>
                </div>
                <div style="display: flex; justify-content: space-between;">
                    <button @click="save_file">save</button>
                    <button @click="relaod_page">reload_page</button>
                </div>
                <div><button @click="openAIFilePick" style="padding: 5 px; margin-left: 5px;">参考ファイルUP</button>
                    <div v-for="(file, ind) in fileDataList" :key="'file_' + ind">{{ getFileName(file) }}, {{ ind
                        }}<button @click="removeFile(file)">×</button></div>
                </div>

                <div>
                    <input type="text" v-model="file_name" placeholder="XXXXのファイル" />
                </div>
                <div class="w-full">
                    <input type="text" class="w-full" v-model="file_detail" placeholder="XXXXに関する説明が記載されているファイルです" />
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
