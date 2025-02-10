<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'

const onClick = async () => {
  const data = await invoke('init')
  console.log(data)
}
</script>

<template>
  <p>This is a test!</p>

  <button :onClick>Invoke</button>
</template>
