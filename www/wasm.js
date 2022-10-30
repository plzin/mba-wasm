// Re-exports everything from the wasm (except the init function)
// but calls init on import.

import init from './mba_wasm.js'

await init()

export * from './mba_wasm.js'