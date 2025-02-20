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

const onchange = useDebounceFn((newTag: string) => {
  const trimmed = newTag.trim()
  emit('change', trimmed === '' ? null : trimmed)
}, 500)
</script>

<template>
  <input
    type="text"
    :title="`Tag for ${friendlyDate(ping.ping)}`"
    :value="ping.tag"
    @onchange="onchange"
  />
</template>
