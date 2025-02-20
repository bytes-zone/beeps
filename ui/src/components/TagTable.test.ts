import { render, screen } from '@testing-library/vue'
import TagTable from './TagTable.vue'
import { expect, test } from 'vitest'
import { friendlyDate } from '@/friendlyDate'

test('when pings are absent, no table should be visible', async () => {
  render(TagTable, {
    props: {
      pings: [],
    },
  })

  expect(await screen.findAllByText('Loading')).not.toBeNull()
})

test('when pings are present, they should be rendered', async () => {
  const ping = new Date()
  const tag = 'Test'

  render(TagTable, {
    props: { pings: [{ ping, tag }] },
  })

  expect(await screen.findAllByText(friendlyDate(ping))).not.toBeNull()
  expect(await screen.findAllByText(tag)).not.toBeNull()
})
