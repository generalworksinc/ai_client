import { createApp } from "vue";
import "./styles.scss";
import App from "./App.vue";
import Main from "./Main.vue";
import Settings from "./Settings.vue";
import Assistants from "./Assistants.vue";
import OpenAIFiles from "./OpenAIFiles.vue";
import Samples from "./Samples.vue";

// add
import { createRouter, createWebHashHistory } from "vue-router";
const routes = [
	{ path: "/", component: Main },
	{ path: "/settings", component: Settings },
	{ path: "/assistants", component: Assistants },
	{ path: "/open_ai_files", component: OpenAIFiles },
	{ path: "/samples", component: Samples },
];

const router = createRouter({
	history: createWebHashHistory(),
	routes, // `routes: routes` の短縮表記
});

// createApp(App).mount("#app");
createApp(App).use(router).mount("#app");
