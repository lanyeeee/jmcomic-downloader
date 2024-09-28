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
  const config = await commands.getConfig();
  console.log(config);
}
</script>

<template>
  <form class="row" @submit.prevent="greet">
    <input id="greet-input" v-model="name" placeholder="Enter a name..."/>
    <button type="submit">Greet</button>
  </form>
  <button @click="test">测试</button>

  <p>{{ greetMsg }}</p>
</template>
