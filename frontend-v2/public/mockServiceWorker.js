/* eslint-disable */
/* tslint:disable */

/**
 * Mock Service Worker.
 * @see https://github.com/mswjs/msw
 * - Please do NOT modify this file.
 * - Please do NOT serve this file on production.
 */

const PACKAGE_VERSION = '2.10.2'
const INTEGRITY_CHECKSUM = '26357c79639bfa20d64c0efca2a87423'
const IS_MOCKED_RESPONSE = Symbol('isMockedResponse')
const activeMocks = new Map()
const ID = 'msw-activation'

function stringifyHeaders(headers) {
  return Object.fromEntries(Array.from(headers.entries()))
}

function getResponse(event, client) {
  const { request } = event
  const clonedRequest = request.clone()

  return new Promise(resolve => {
    client.postMessage({
      type: 'request',
      id: event.resultingClientId,
      url: request.url,
      method: request.method,
      headers: stringifyHeaders(request.headers),
      body: clonedRequest.text(),
    })

    function handleMessage(event) {
      if (event.data && event.data.type === 'response') {
        resolve(event.data)
        self.removeEventListener('message', handleMessage)
      }
    }

    self.addEventListener('message', handleMessage)
  })
}

self.addEventListener('install', function (event) {
  self.skipWaiting()
})

self.addEventListener('activate', function (event) {
  event.waitUntil(self.clients.claim())
})

self.addEventListener('message', async function (event) {
  const clientId = event.source.id

  if (!clientId || !event.data) {
    return
  }

  const message = event.data

  switch (message.type) {
    case 'MOCK_ACTIVATE': {
      activeMocks.set(clientId, message)
      event.source.postMessage({
        type: 'MOCK_ACTIVATE',
        payload: { id: ID, status: 'activated' },
      })
      break
    }

    case 'MOCK_DEACTIVATE': {
      activeMocks.delete(clientId)
      event.source.postMessage({
        type: 'MOCK_DEACTIVATE',
        payload: { id: ID, status: 'deactivated' },
      })
      break
    }

    case 'CLIENT_CLOSED': {
      activeMocks.delete(clientId)
      break
    }

    case 'INTEGRITY_CHECK_REQUEST': {
      event.source.postMessage({
        type: 'INTEGRITY_CHECK_RESPONSE',
        payload: { 
          checksum: INTEGRITY_CHECKSUM,
          version: PACKAGE_VERSION 
        },
      })
      break
    }
  }
})

self.addEventListener('fetch', function (event) {
  const { request } = event
  const requestClone = request.clone()
  const getOriginalResponse = () => fetch(requestClone)

  const mockResponse = activeMocks.get(event.clientId)
  if (!mockResponse || !mockResponse.active) {
    return
  }

  event.respondWith(
    new Promise(async (resolve) => {
      const client = await self.clients.get(event.clientId)

      if (client) {
        const message = await getResponse(event, client)

        switch (message.type) {
          case 'MOCK_RESPONSE': {
            const response = new Response(message.payload.body, {
              status: message.payload.status,
              statusText: message.payload.statusText,
              headers: message.payload.headers,
            })

            Object.defineProperty(response, IS_MOCKED_RESPONSE, {
              value: true,
              enumerable: false,
            })

            resolve(response)
            break
          }

          case 'PASSTHROUGH': {
            resolve(getOriginalResponse())
            break
          }

          default: {
            resolve(getOriginalResponse())
          }
        }
      } else {
        resolve(getOriginalResponse())
      }
    })
  )
})