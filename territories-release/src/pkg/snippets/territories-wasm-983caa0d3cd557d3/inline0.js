
export async function ortBridgeInit(bytes) {
  if (typeof globalThis.__territoriesOrtInit !== 'function') {
    throw new Error('ONNX bridge not initialized in worker');
  }
  return await globalThis.__territoriesOrtInit(bytes);
}
export async function ortBridgeInfer(flatTensor, batchSize) {
  if (typeof globalThis.__territoriesOrtInfer !== 'function') {
    throw new Error('ONNX inference bridge not initialized in worker');
  }
  return await globalThis.__territoriesOrtInfer(flatTensor, batchSize);
}
