import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

type Doc = {
  pings: string[]
  tags: Record<string, { value: string | null }>
}

export type Ping = {
  ping: Date
  tag: string | null
}

export const store = ref<Ping[]>([])

await invoke<Doc>('init').then((doc) => {
  for (const ping of doc.pings) {
    store.value.unshift({
      ping: new Date(ping),
      tag: doc.tags[ping]?.value,
    })
  }
})
