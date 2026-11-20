import { HalfFloatType, UnsignedByteType, WebGPURenderer } from 'three/webgpu'
import { GPUFeatureName } from 'three/src/renderers/webgpu/utils/WebGPUConstants.js'

async function requestWorldDevice() {
    try {
        const adapter = await navigator.gpu?.requestAdapter({ featureLevel: 'core', powerPreference: 'high-performance' })
        // The world's shadow shaders and texture-array views require core
        // WebGPU. Older WebViews may ignore featureLevel in the request.
        if (!adapter?.features.has(GPUFeatureName.CoreFeaturesAndLimits)) return
        const requiredFeatures = Object.values(GPUFeatureName).filter(feature => adapter.features.has(feature))
        return await adapter.requestDevice({ requiredFeatures })
    } catch {
        // Adapter/device acquisition can also be denied by the browser.
    }
}

export async function createWorldRenderer(canvas, antialias) {
    const device = await requestWorldDevice()
    if (device) return new WebGPURenderer({ canvas, device, antialias })

    const context = canvas.getContext('webgl2', {
        antialias, alpha: true, depth: true, stencil: false, powerPreference: 'high-performance',
    })
    if (!context) throw new Error('WebGL 2 is unavailable')
    const hdr = context.getExtension('EXT_color_buffer_float') || context.getExtension('EXT_color_buffer_half_float')
    return new WebGPURenderer({
        canvas, context, antialias, forceWebGL: true, outputBufferType: hdr ? HalfFloatType : UnsignedByteType,
    })
}
