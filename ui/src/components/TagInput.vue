<script setup lang="ts">
import type { PingWithTag } from '@/store'
import { useDebounceFn } from '@vueuse/core'
import { friendlyDate } from '@/friendlyDate'

defineProps<{
  ping: PingWithTag
}>()

const emit = defineEmits<{
  change: [value: string | null]
}>()

const onchange = useDebounceFn((ev: Event) => {
  const el = ev.target as HTMLInputElement
  const trimmed = el.value.trim()
  emit('change', trimmed === '' ? null : trimmed)
}, 250)
</script>

<template>
  <input
    type="text"
    :title="`Tag for ${friendlyDate(ping.ping)}`"
    :value="ping.tag"
    @change="onchange"
  />
</template>
