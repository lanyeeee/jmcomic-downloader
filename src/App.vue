<script setup lang="ts">
// This starter template is using Vue 3 <script setup> SFCs
// Check out https://vuejs.org/api/sfc-script-setup.html#script-setup
import AppContent from "./AppContent.vue";
</script>

<template>
  <n-modal-provider>
    <n-notification-provider placement="bottom-right">
      <n-message-provider>
        <app-content/>
      </n-message-provider>
    </n-notification-provider>
  </n-modal-provider>
</template>
