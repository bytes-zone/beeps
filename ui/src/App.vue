<script setup lang="ts">
import { current, error } from './store'
import TagTable from './components/TagTable.vue'

function dismissError() {
  error.value = null
}
</script>

<template>
  <template v-if="error">
    <span>Error:</span> {{ error }} <button @click="dismissError">X</button>
  </template>
  <TagTable :pings="current" />
</template>
