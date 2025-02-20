import { render, screen } from '@testing-library/vue'
import TagInput from './TagInput.vue'
import { describe, expect, test } from 'vitest'
import { friendlyDate } from '@/friendlyDate'

test('title should be rendered', async () => {
  const ping = new Date()
  const tag = 'Test'

  render(TagInput, {
    props: { ping: { ping, tag } },
  })

  expect(screen.getByTitle(`Tag for ${friendlyDate(ping)}`)).not.toBeNull()
})

describe('value should be rendered', () => {
  function renderInput(tag: string | null): HTMLInputElement {
    const ping = new Date()

    render(TagInput, {
      props: { ping: { ping, tag } },
    })

    return screen.getByTitle(`Tag for ${friendlyDate(ping)}`)
  }

  test('when null', () => {
    const el = renderInput(null)

    expect(el.value).toEqual('')
  })

  test('when present', () => {
    const el = renderInput('Test')

    expect(el.value).toEqual('Test')
  })
})
