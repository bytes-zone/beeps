<script setup lang="ts">
import { current, error } from './store'
import TagTable from './components/TagTable.vue'
</script>

<template>
  <template v-if="error"> <span>Error:</span> {{ error }} </template>
  <TagTable :pings="current" />
</template>
