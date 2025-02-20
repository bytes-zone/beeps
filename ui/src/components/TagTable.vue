<script setup lang="ts">
import { type PingWithTag } from '@/store'
import { friendlyDate } from '@/friendlyDate'
import TagInput from './TagInput.vue'

defineProps<{
  pings: PingWithTag[]
}>()

defineEmits<{
  tag: [ping: Date, tag: string | null]
}>()
</script>

<template>
  <p v-if="pings.length == 0">Loading</p>
  <table v-else>
    <thead>
      <tr>
        <th scope="col">Ping</th>
        <th scope="col">Tag</th>
      </tr>
    </thead>
    <tbody>
      <tr v-for="ping in pings" :key="ping.ping.toString()">
        <td scope="row">
          {{ friendlyDate(ping.ping) }}
        </td>
        <td><TagInput :ping="ping" @change="(tag) => $emit('tag', ping.ping, tag)" /></td>
      </tr>
    </tbody>
  </table>
</template>
