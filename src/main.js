import { createApp } from "vue";
import "./styles.scss";
import App from "./App.vue";
import Main from "./Main.vue";
import Settings from "./Settings.vue";


// add 
import { createRouter, createWebHashHistory } from 'vue-router'
const routes = [
    { path: '/', component: Main },
    { path: '/settings', component: Settings }
]

const router = createRouter({
    history: createWebHashHistory(),
    routes // `routes: routes` の短縮表記
});

// createApp(App).mount("#app");
createApp(App).use(router).mount("#app");