import { ref } from 'vue'
import { commands, type PingWithTag } from './bindings'

export const store = ref<PingWithTag[]>([])

export const error = ref<string | null>(null)

async function refreshDocument() {
  store.value = (await commands.document()).pings
}

async function schedulePings() {
  const result = await commands.schedulePings()

  // TODO: this may not be in the right order
  if (result.status == 'ok') {
    store.value.unshift(...result.data)
  } else {
    error.value = `could not schedule new pings: ${result.error}`
  }
}

// We use an IIFE because we want to maximize OS compatibility and Safari only
// supports top-level await back to ~2021.
;(async () => {
  setInterval(schedulePings, 10000)

  await refreshDocument()
  await schedulePings()
})()
