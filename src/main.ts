import { createApp } from "vue";
import App from "./App.vue";
import { invoke } from "@tauri-apps/api/core";

// Disable default right-click context menu
// document.addEventListener("contextmenu", (event) => {
//   event.preventDefault();
// });

createApp(App).mount("#app");
invoke("show_main_window")
    .then(() => {
        console.log("🚪 show_main_window invoked successfully");
    })
    .catch((err) => {
        console.error("❗ show_main_window error:", err);
    });