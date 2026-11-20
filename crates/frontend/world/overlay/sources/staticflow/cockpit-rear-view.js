import { Color, Euler, LinearFilter, LinearSRGBColorSpace, PerspectiveCamera, Quaternion, RenderTarget, Vector3 } from 'three/webgpu'
import { createRearViewInstances } from './rear-view-instances.js'

// A camera fixed to the rearward sightline of the cab. The glass reverses
// the horizontal UV; doing that in the projection would invert winding too.
export function createRearView(game, setTexture, cockpitGroup) {
    const target = new RenderTarget(192, 64, {
        minFilter: LinearFilter, magFilter: LinearFilter, colorSpace: LinearSRGBColorSpace,
        generateMipmaps: false, samples: 0, depthBuffer: true, stencilBuffer: false,
    })
    target.texture.name = 'rear-view-road'
    const camera = new PerspectiveCamera(34, target.width / target.height, .18, 36)
    camera.name = 'rear-view-camera'
    const rearFacing = new Quaternion().setFromEuler(new Euler(-.09, Math.PI / 2, 0, 'YXZ'))
    const rearward = new Vector3()
    const lastPosition = new Vector3()
    const lastRotation = new Quaternion()
    const lastLight = new Color()
    let lastRain = 0
    let lastCapture = -Infinity
    let lastVisit
    let lastMainFrame
    let frameSeconds = 1 / 60
    let framesSinceCapture = 0

    const { renderer } = game.rendering
    const { world } = game
    const floor = world.floor
    const scene = game.scene
    const instances = createRearViewInstances(world, scene)
    // The water surface takes a screen/depth copy for its refraction. The
    // underlying floor still includes the water-depth colour in the mirror.
    // Windscreen-space streaks, front-streamed particles and the mirror's own
    // sampler must not enter this pass.
    const excluded = [
        cockpitGroup, game.view.speedLines.mesh, game.overlay.mesh, world.waterSurface.mesh, world.grass.mesh,
        world.rain.mesh, world.snow.mesh, world.leaves.mesh, world.windLines.mesh,
    ].filter(Boolean)
    const areas = Object.values(world.areas).filter(area => area.frustum && area.objects?.hideable?.length)
    const hidden = [], revealed = []

    function hide(object) {
        if (object?.visible) { hidden.push(object); object.visible = false }
    }

    function update() {
        if (!cockpitGroup.visible || document.hidden) {
            lastVisit = undefined
            return
        }
        const now = game.ticker.elapsed
        const frame = renderer.info.frame
        if (lastVisit === now) return
        // info.frame advances once per animation frame, even with many scene
        // passes. Skipped main frames mean the cockpit has just become visible.
        const entering = lastVisit === undefined || frame > lastMainFrame + 1
        const gap = entering ? 0 : now - lastVisit
        lastVisit = now
        lastMainFrame = frame
        if (gap > 0) {
            // Ticker.delta is capped for physics. Measure real display frames
            // here, limiting one stall's influence on the moving average.
            frameSeconds += .22 * (Math.min(1,Math.max(1 / 240,gap)) - frameSeconds)
        }
        framesSinceCapture++
        const minFrames = gap >= .10 || frameSeconds >= .05 ? 8 : frameSeconds >= .034 ? 4 : 1
        const interval = frameSeconds > .024 || game.quality.level > 0 ? .2 : .14
        if (!entering && (framesSinceCapture < minFrames || now - lastCapture < interval)) return
        const chassis = game.physicalVehicle
        const light = game.dayCycles.properties.lightColor.value
        const rain = game.weather.rain.value
        const moved = lastPosition.distanceToSquared(chassis.position) > .035 ** 2 ||
            Math.abs(lastRotation.dot(chassis.quaternion)) < .99999
        const atmosphereChanged = Math.max(Math.abs(light.r - lastLight.r), Math.abs(light.g - lastLight.g),
            Math.abs(light.b - lastLight.b), Math.abs(rain - lastRain)) > .015
        // A parked driver can still see rolling crates or a changing sky.
        if (!entering && !moved && now - lastCapture < (atmosphereChanged ? .75 : 1.1)) return

        camera.position.set(-.50,.48,-.09).applyQuaternion(chassis.quaternion).add(chassis.position)
        camera.quaternion.copy(chassis.quaternion).multiply(rearFacing)
        camera.getWorldDirection(rearward)
        const length = Math.hypot(rearward.x,rearward.z)
        if (length > .001) { rearward.x /= length; rearward.z /= length }

        const previousTarget = renderer.getRenderTarget()
        const previousFace = renderer.getActiveCubeFace()
        const previousMip = renderer.getActiveMipmapLevel()
        const previousMRT = renderer.getMRT()
        const updateWorldMatrices = scene.matrixWorldAutoUpdate
        const shadow = game.lighting.light.shadow
        const autoShadow = shadow.autoUpdate, pendingShadow = shadow.needsUpdate
        const tile = floor.mesh.position
        const oldX = tile.x, oldZ = tile.z
        try {
            for (const object of excluded) hide(object)
            // The garage can swap the active body and its boost trails.
            const { parts, antenna, boostTrails } = world.visualVehicle
            hide(parts.chassis)
            hide(antenna?.head)
            hide(boostTrails?.left?.mesh)
            hide(boostTrails?.right?.mesh)
            // The normal streaming pass hides exhibit buildings outside the
            // windscreen. Borrow only those clusters intersecting the rear cone.
            for (const area of areas) {
                if (area.frustum.isIn !== false) continue
                const { position, radius } = area.frustum
                const dx = position.x - camera.position.x, dz = position.y - camera.position.z
                const along = dx * rearward.x + dz * rearward.z
                const across = Math.abs(dx * rearward.z - dz * rearward.x)
                if (along < -radius || along > camera.far + radius || across > Math.max(0,along) + radius * 1.45) continue
                for (const object of area.objects.hideable) {
                    if (!object.visible) { revealed.push(object); object.visible = true }
                }
            }
            const ahead = Math.min(9,floor.size * .24)
            tile.x = Math.round((camera.position.x + ahead * rearward.x) / floor.cellSize) * floor.cellSize
            tile.z = Math.round((camera.position.z + ahead * rearward.z) / floor.cellSize) * floor.cellSize
            // The main pass has just updated every world matrix, including
            // streamed-out buildings. Only the rear floor tile moved since.
            floor.mesh.updateWorldMatrix(false, false)
            scene.matrixWorldAutoUpdate = false
            instances.show(camera, hide)
            // Pinned Three's WebGPU renderer (including its WebGL2 backend)
            // tests these per-light flags for each camera. Reuse the existing
            // shadow texture and leave the main camera's pending update intact.
            shadow.autoUpdate = false
            shadow.needsUpdate = false
            renderer.setMRT(null)
            renderer.setRenderTarget(target)
            renderer.render(game.scene,camera)
        } finally {
            renderer.setRenderTarget(previousTarget,previousFace,previousMip)
            renderer.setMRT(previousMRT)
            instances.group.visible = false
            shadow.autoUpdate = autoShadow
            shadow.needsUpdate = pendingShadow
            tile.x = oldX
            tile.z = oldZ
            floor.mesh.updateWorldMatrix(false, false)
            scene.matrixWorldAutoUpdate = updateWorldMatrices
            for (const object of revealed) object.visible = false
            for (const object of hidden) object.visible = true
            revealed.length = hidden.length = 0
        }
        if (lastCapture === -Infinity) setTexture(target.texture)
        framesSinceCapture = 0
        lastCapture = now
        lastPosition.copy(chassis.position)
        lastRotation.copy(chassis.quaternion)
        lastLight.copy(light)
        lastRain = rain
    }
    return { update }
}
