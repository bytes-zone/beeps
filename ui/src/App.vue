<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'

const onClick = async () => {
  await invoke('my_custom_command')
}
</script>

<template>
  <p>This is a test!</p>

  <button :onClick>Invoke</button>
</template>
