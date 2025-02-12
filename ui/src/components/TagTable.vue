<script setup lang="ts">
import { type Ping } from '../store.js'

defineProps<{
  pings: Ping[]
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
