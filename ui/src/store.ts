import { ref } from 'vue'
import { commands, type PingWithTag } from './bindings'

export const store = ref<PingWithTag[]>([])

// We use an IIFE because we want to maximize OS compatibility and Safari only
// supports top-level await back to ~2021.
;(async () => {
  await commands.init().then((doc) => {
    store.value = doc.pings
  })
})()
