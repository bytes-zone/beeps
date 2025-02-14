<script setup lang="ts">
import { type PingWithTag } from '@/bindings'

defineProps<{
  pings: PingWithTag[]
}>()
</script>

<template>
  <table>
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
