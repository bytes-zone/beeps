import { ref } from 'vue'
import { commands, type PingWithTag as RawPingWithTag } from './bindings'

export type PingWithTag = Omit<RawPingWithTag, 'ping'> & { ping: Date }

export const store = ref<PingWithTag[]>([])

export const error = ref<string | null>(null)

async function refreshDocument() {
  store.value = (await commands.document()).pings.map((ping) => ({
    ...ping,
    ping: new Date(ping.ping),
  }))
}

async function schedulePings() {
  const result = await commands.schedulePings()

  // TODO: this may not be in the right order
  if (result.status == 'ok') {
    for (const ping of result.data) {
      store.value.unshift({ ...ping, ping: new Date(ping.ping) })
    }
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
