<script setup lang="ts">
import {ref} from "vue";
import {commands} from "../bindings.ts";

const greetMsg = ref("");
const name = ref("");

async function greet() {
  // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
  greetMsg.value = await commands.greet(name.value);
}

async function test() {
  const result = await commands.getAlbum(456688);
  if (result.status === "error") {
    console.error(result.error);
    return;
  }
  console.log(result.data);
}
</script>

<template>
  <form class="row" @submit.prevent="greet">
    <input id="greet-input" v-model="name" placeholder="Enter a name..."/>
    <button type="submit">Greet</button>
  </form>
  <n-button @click="test">测试</n-button>

  <p class="text-red">{{ greetMsg }}</p>
</template>
