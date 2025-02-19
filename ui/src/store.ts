import { ref } from 'vue'
import { commands, type PingWithTag as RawPingWithTag } from './bindings'

export type PingWithTag = Omit<RawPingWithTag, 'ping'> & { ping: Date }

// The set of tags we want the user to see in the UI
export const current = ref<PingWithTag[]>([])

// Future pings we want to hide (for now)
export const future = ref<PingWithTag[]>([])

// Any errors
export const error = ref<string | null>(null)

function storePing(ping: RawPingWithTag) {
  const converted = { ...ping, ping: new Date(ping.ping) }

  if (converted.ping <= new Date()) {
    current.value.unshift(converted)
  } else {
    future.value.push(converted)
  }
}

async function refreshDocument() {
  const document = await commands.document()

  document.pings.forEach(storePing)
}

async function schedulePings() {
  const result = await commands.schedulePings()

  if (result.status == 'ok') {
    result.data.forEach(storePing)
  } else {
    error.value = `could not schedule new pings: ${result.error}`
  }
}

function movePingsFromFuture() {
  const now = new Date()

  future.value = future.value.filter((ping) => {
    if (ping.ping <= now) {
      current.value.unshift(ping)
      return false
    }
    return true
  })
}

// We use an IIFE because we want to maximize OS compatibility and Safari only
// supports top-level await back to ~2021.
;(async () => {
  setInterval(schedulePings, 10000)
  setInterval(movePingsFromFuture, 1000)

  await refreshDocument()
  await schedulePings()
})()
