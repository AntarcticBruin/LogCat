<script setup lang="ts">
const selectedHostId = defineModel<string | null>("selectedHostId", { required: true });
const hostName = defineModel<string>("hostName", { required: true });
const host = defineModel<string>("host", { required: true });
const port = defineModel<number>("port", { required: true });
const username = defineModel<string>("username", { required: true });
const authType = defineModel<"password" | "key">("authType", { required: true });
const password = defineModel<string>("password", { required: true });
const keyPath = defineModel<string>("keyPath", { required: true });
const passphrase = defineModel<string>("passphrase", { required: true });

defineProps<{
  isOpen: boolean;
}>();

const emit = defineEmits<{
  (event: "close"): void;
  (event: "save"): void;
  (event: "pick-key-path"): void;
}>();
</script>

<template>
  <div v-if="isOpen" class="modal-overlay" @click.self="emit('close')">
    <div class="modal-content">
      <div class="modal-header">
        <h3>{{ selectedHostId ? "Edit Host" : "Add Host" }}</h3>
        <button class="close-modal" @click="emit('close')">×</button>
      </div>
      <div class="modal-body">
        <div class="form-group">
          <label>Name</label>
          <input v-model="hostName" placeholder="Production Server" />
        </div>
        <div class="form-group">
          <label>Host Address</label>
          <input v-model="host" placeholder="192.168.1.100" />
        </div>
        <div class="form-row">
          <div class="form-group">
            <label>Port</label>
            <input v-model.number="port" type="number" />
          </div>
          <div class="form-group">
            <label>Username</label>
            <input v-model="username" />
          </div>
        </div>
        <div class="form-group">
          <label>Authentication</label>
          <div class="auth-switch">
            <button
              type="button"
              class="auth-switch-btn"
              :class="{ active: authType === 'password' }"
              @click="authType = 'password'"
            >
              Password
            </button>
            <button
              type="button"
              class="auth-switch-btn"
              :class="{ active: authType === 'key' }"
              @click="authType = 'key'"
            >
              SSH Key
            </button>
          </div>
        </div>
        <div class="form-group">
          <template v-if="authType === 'password'">
            <label>Password</label>
            <input v-model="password" type="password" />
          </template>
          <template v-else>
            <label>Private Key Path</label>
            <div class="file-picker">
              <input v-model="keyPath" placeholder="Select a private key file" />
              <button type="button" class="btn btn-outline file-picker-btn" @click="emit('pick-key-path')">
                Browse
              </button>
            </div>
          </template>
        </div>
        <div v-if="authType === 'key'" class="form-group">
          <label>Passphrase</label>
          <input v-model="passphrase" type="password" placeholder="Optional" />
        </div>
      </div>
      <div class="modal-footer">
        <button class="btn btn-outline" @click="emit('close')">Cancel</button>
        <button class="btn btn-primary" @click="emit('save')">Save Host</button>
      </div>
    </div>
  </div>
</template>
