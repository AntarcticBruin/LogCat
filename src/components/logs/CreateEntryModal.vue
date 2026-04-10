<script setup lang="ts">
const isOpen = defineModel<boolean>("isOpen", { required: true });
const name = defineModel<string>("name", { required: true });

defineProps<{
  kindLabel: string;
  currentPath: string;
  saving: boolean;
}>();

const emit = defineEmits<{
  (event: "close"): void;
  (event: "confirm"): void;
}>();
</script>

<template>
  <div v-if="isOpen" class="modal-overlay" @click.self="emit('close')">
    <div class="modal-content action-modal">
      <div class="modal-header">
        <div>
          <h3>New {{ kindLabel }}</h3>
          <div class="modal-subtitle">{{ currentPath }}</div>
        </div>
        <button class="close-modal" @click="emit('close')">×</button>
      </div>
      <div class="modal-body">
        <div class="form-group">
          <label>{{ kindLabel }} Name</label>
          <input v-model="name" :placeholder="`Enter ${kindLabel.toLowerCase()} Name`" @keyup.enter="emit('confirm')" />
        </div>
      </div>
      <div class="modal-footer">
        <button class="btn btn-outline" :disabled="saving" @click="emit('close')">Cancel</button>
        <button class="btn btn-primary" :disabled="saving || !name.trim()" @click="emit('confirm')">
          {{ saving ? "Creating..." : "Create" }}
        </button>
      </div>
    </div>
  </div>
</template>
