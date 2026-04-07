import { createApp } from "vue";
import App from "./App.vue";
import {invoke} from "@tauri-apps/api/core";

createApp(App).mount("#app");
invoke("show_main_window")
    .then(() => {
        console.log("🚪 show_main_window invoked successfully");
    })
    .catch((err) => {
        console.error("❗ show_main_window error:", err);
    });