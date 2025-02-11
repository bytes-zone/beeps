import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export const store = ref(await invoke('init'))
