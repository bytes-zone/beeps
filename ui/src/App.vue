<script setup lang="ts">
import { store } from './store'
import TagTable from './components/TagTable.vue'
</script>

<template>
  <TagTable :pings="store" />
</template>
