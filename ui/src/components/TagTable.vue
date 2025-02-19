<script setup lang="ts">
import { type PingWithTag } from '@/store'

defineProps<{
  pings: PingWithTag[]
}>()
</script>

<template>
  <p v-if="pings.length == 0">Loading</p>
  <table v-else>
    <thead>
      <tr>
        <th>Ping</th>
        <th>Tag</th>
      </tr>
    </thead>
    <tbody>
      <tr v-for="ping in pings" :key="ping.ping.toString()">
        <td>{{ ping.ping }}</td>
        <td>{{ ping.tag }}</td>
      </tr>
    </tbody>
  </table>
</template>
