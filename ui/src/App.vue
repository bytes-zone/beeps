<script setup lang="ts">
import { current } from './store'
import TagTable from './components/TagTable.vue'
</script>

<template>
  <TagTable :pings="current" />
</template>
