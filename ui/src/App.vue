<script setup lang="ts">
import { store } from './store'
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
      <tr v-for="ping in store" :key="ping.ping.toString()">
        <td>{{ ping.ping }}</td>
        <td>{{ ping.tag }}</td>
      </tr>
    </tbody>
  </table>
</template>
