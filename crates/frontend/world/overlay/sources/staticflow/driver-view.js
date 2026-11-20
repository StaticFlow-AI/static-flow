import { Euler, MathUtils, Quaternion, Vector2, Vector3 } from 'three/webgpu'
import { createDriverCockpit } from './driver-cockpit.js'
import { createRearView } from './cockpit-rear-view.js'

export class DriverView {
    constructor(view) {
        this.view = view
        this.enabled = false
        this.weight = 0
        this.perspective = 0
        this.switchFrom = 0
        this.switchStart = null
        this.cockpit = null
        this.steeringAngle = 0
        this.motionTime = null
        this.previousVelocity = new Vector3()
        this.acceleration = new Vector3()
        this.headOffset = new Vector3()
        this.targetOffset = new Vector3()
        this.headAngles = new Euler(0, 0, 0, 'YXZ')
        this.headRotation = new Quaternion()
        this.inverseCamera = new Quaternion()
        this.seat = new Vector3()
        this.telemetry = { speedKmh: 0, drive: 'N', throttle: 0, brake: 0 }
        this.hideExterior = false
        this.eye = new Vector3()
        this.heading = new Vector3(1, 0, 0)
        this.target = new Vector3()
        this.touchOrigin = new Vector2()
        this.touchAngle = 0
        this.dialogOpen = false
        document.addEventListener('toggle', event => {
            if (event.target.tagName === 'DIALOG') this.dialogOpen = Boolean(document.querySelector('dialog[open]'))
        }, true)
    }

    setEnabled(enabled) {
        this.enabled = enabled
        this.switchFrom = this.perspective
        this.switchStart = this.view.game.ticker.elapsed
        this.resetMotion()
        if (enabled) this.view.focusPoint.isTracking = true
        const nipple = this.view.game.inputs.nipple
        nipple.active = false
        nipple.inRadiusLow = false
        nipple.progress = 0
    }

    resetMotion() {
        this.motionTime = null
        this.headOffset.set(0, 0, 0)
        this.headAngles.set(0, 0, 0, 'YXZ')
        this.headRotation.identity()
    }

    updateMotion(car, game) {
        const { elapsed, deltaScaled } = game.ticker
        const velocity = car.chassis.physical.body.linvel()
        const speed = Math.hypot(velocity.x, velocity.z)
        const forwardSpeed = car.forward.dot(velocity)
        const pedal = game.player.accelerating
        const braking = game.player.braking || Math.abs(pedal) > .1
        const maxBrakeImpulse = car.brakeAmplitude * deltaScaled
        this.telemetry.speedKmh = speed * 3.6
        this.telemetry.daylight = game.dayCycles.properties.lightColor.value
        this.telemetry.rain = game.weather.rain.value
        this.telemetry.elapsed = elapsed
        const direction = Math.abs(forwardSpeed) > .3 ? forwardSpeed : pedal
        this.telemetry.drive = direction < -.1 ? 'R' : direction > .1 ? 'D' : 'N'
        this.telemetry.brake = braking && maxBrakeImpulse > 0
            ? MathUtils.clamp(car.controller.wheelBrake(0) / maxBrakeImpulse, 0, 1) : 0
        this.telemetry.throttle = this.telemetry.brake > 0 ? 0 : Math.min(1, Math.abs(pedal))

        const wheelAngle = car.controller.wheelSteering(0)
        if (this.motionTime === null) {
            this.steeringAngle = wheelAngle
        } else {
            const dt = elapsed - this.motionTime
            this.steeringAngle = MathUtils.damp(this.steeringAngle, wheelAngle, 1 / .085, dt)
            if (Math.abs(this.steeringAngle - wheelAngle) < .0005) this.steeringAngle = wheelAngle

            // Rapier velocities use simulation seconds; the body and steering
            // settle over wall-clock seconds even when the frame rate changes.
            // Ignore small contact-solver corrections while resting on the road.
            const moving = speed > .2 || Math.abs(velocity.y) > .35 || this.previousVelocity.lengthSq() > .14
            this.targetOffset.set(0, 0, 0)
            if (moving && deltaScaled > 0) {
                this.acceleration.subVectors(velocity, this.previousVelocity).divideScalar(deltaScaled)
                // Local axes are fore/aft, vertical, then lateral. In a left
                // turn the chassis accelerates left and the driver leans right.
                this.targetOffset.set(
                    MathUtils.clamp(-this.acceleration.dot(car.forward) * .0036, -.041, .041),
                    MathUtils.clamp(-this.acceleration.dot(car.upward) * .0018, -.023, .023),
                    MathUtils.clamp(-this.acceleration.dot(car.sideward) * .0044, -.046, .046),
                )
            }
            this.headOffset.lerp(this.targetOffset, -Math.expm1(-dt / .11))
            if (this.headOffset.lengthSq() < 1e-9 && this.targetOffset.lengthSq() === 0) this.headOffset.set(0, 0, 0)
        }
        this.previousVelocity.copy(velocity)
        this.motionTime = elapsed
        this.headAngles.set(-this.headOffset.x * .56, this.steeringAngle * Math.min(speed / 8, 1) * .085,
            -this.headOffset.z * .62, 'YXZ')
        this.headRotation.setFromEuler(this.headAngles)
    }

    updatePose() {
        const { camera, defaultCamera, cinematic, game, focusPoint } = this.view
        const car = game.physicalVehicle
        const filters = game.inputs.filters
        const panelOpen = this.dialogOpen || filters.has('modal') || filters.has('menu')
        // The launch animation enables driving while its reveal shader still
        // masks the road. Keep the driver's selection until the grid is gone.
        const ready = game.reveal?.step === 2 && focusPoint.isTracking && !this.view.freeMode.enabled && !panelOpen
        if (!ready || !car) {
            this.perspective = 0
            this.switchStart = null
        } else if (this.switchStart === null) {
            this.perspective = Number(this.enabled)
        } else {
            const t = MathUtils.smoothstep(game.ticker.elapsed - this.switchStart, 0, .24)
            this.perspective = MathUtils.lerp(this.switchFrom, Number(this.enabled), t)
            if (t === 1) this.switchStart = null
        }
        this.weight = this.perspective * (1 - cinematic.progress)
        // Use the existing render-pass camera. Cursor picking, audio and the
        // world all read its exact pose. Keep the independent overview camera
        // intact for teleporting, resizing and returning from an exhibit.
        if (this.weight > 0) {
            this.updateMotion(car, game)
            const forward = car.forward
            if (Math.hypot(forward.x, forward.z) > .02)
                this.heading.set(forward.x, 0, forward.z).normalize()
            const pitch = MathUtils.clamp(forward.y, -.42, .42) - .09
            // The chassis uses +X forward and -Z for the left seat. Attach
            // the eye to its rigid body, including on jumps and rollovers.
            this.seat.set(-.06, .43, -.27).applyQuaternion(car.quaternion).add(car.position)
            this.eye.copy(this.headOffset).applyQuaternion(car.quaternion).add(this.seat)
            camera.position.copy(this.eye)
            this.target.copy(camera.position).add(this.heading)
            this.target.y += pitch
            camera.lookAt(this.target)
            // Feel the suspension when crossing a curb without rolling the
            // entire horizon on a hard collision.
            const bank = Math.atan2(car.sideward.y, car.upward.y)
            camera.rotateZ(MathUtils.clamp(bank * .6, -.34, .34))
            camera.quaternion.multiply(this.headRotation)
            if (cinematic.progress > 0) {
                camera.position.lerp(cinematic.dummy.position, cinematic.progress)
                camera.quaternion.slerp(cinematic.dummy.quaternion, cinematic.progress)
            }
            if (this.perspective < 1) {
                camera.position.lerp(defaultCamera.position, 1 - this.perspective)
                camera.quaternion.slerp(defaultCamera.quaternion, 1 - this.perspective)
            }
        }
        if (!this.weight && this.motionTime !== null) this.resetMotion()
        if (game.rendering?.cheapDOFPass) game.rendering.cheapDOFPass.overview.value = 1 - this.weight
        const fov = MathUtils.lerp(defaultCamera.fov, camera.aspect < 1 ? 78 : 68, this.weight)
        if (camera.fov !== fov) {
            camera.fov = fov
            camera.updateProjectionMatrix()
        }

        // During an exhibit cutaway the camera travels outside the vehicle.
        // Let the dashboard leave with the seat; reveal the opaque exterior
        // once the moving camera has cleared the cab.
        const eyeDistanceSq = this.weight > 0 ? camera.position.distanceToSquared(this.eye) : Infinity
        this.hideExterior = eyeDistanceSq < 1.2 * 1.2
        const seated = eyeDistanceSq < .06 * .06
        if (seated) {
            if (!this.cockpit) {
                this.cockpit = createDriverCockpit()
                camera.add(this.cockpit.group)
                this.rearView = createRearView(game, this.cockpit.setRearViewTexture, this.cockpit.group)
                car.events.on('teleport', () => this.resetMotion())
            }
            // The cabin stays on the sprung chassis while the eye can move
            // inside it. Transform the fixed seat back into camera space so
            // the wheel and dashboard respond together, even on a bank.
            const { group } = this.cockpit
            this.inverseCamera.copy(camera.quaternion).invert()
            group.position.copy(this.seat).sub(camera.position).applyQuaternion(this.inverseCamera)
            group.quaternion.copy(this.headRotation).invert()
            this.cockpit.update(this.steeringAngle, camera.aspect, camera.fov, this.telemetry)
        }
        if (this.cockpit) this.cockpit.group.visible = seated
    }

    updateMirror() {
        if (this.cockpit?.group.visible) this.rearView.update()
    }

    updateFootprint() {
        if (!this.weight) return
        const { optimalArea: area, game } = this.view
        // Keep the same streamed ground/grass/shadow tile sizes. Move their
        // center toward the windscreen so the road is present up to the fog.
        const radius = area.radius
        const car = game.physicalVehicle.position
        this.target.set(car.x + this.heading.x * radius * .44, 0, car.z + this.heading.z * radius * .44)
        area.position.lerp(this.target, this.weight)
        for (let i = 0; i < 4; i++) {
            const corner = area.quad2[i].offseted
            corner.x = MathUtils.lerp(corner.x, this.target.x + (i < 2 ? radius : -radius), this.weight)
            corner.y = MathUtils.lerp(corner.y, this.target.z + (i === 0 || i === 3 ? radius : -radius), this.weight)
        }
    }

    updateTouch(nipple, pointer, start) {
        if (start) this.touchOrigin.set(pointer.current.x, pointer.current.y)
        const dx = pointer.current.x - this.touchOrigin.x
        const dy = this.touchOrigin.y - pointer.current.y
        const distance = Math.hypot(dx, dy)
        const travel = MathUtils.clamp(Math.min(this.view.game.viewport.width, this.view.game.viewport.height) * .14, 45, 80)
        nipple.progress = MathUtils.clamp((distance - 5) / travel, 0, 1)
        nipple.uniforms.progress.value = nipple.progress
        if (start) nipple.inRadiusLow = true
        else if (nipple.progress > 0) nipple.inRadiusLow = false
        // The same screen-space drag continues to steer relative to the
        // windscreen as the car turns. Dragging back selects reverse.
        this.touchAngle = Math.atan2(dx, dy)
    }
}

// The overview foliage is made of parallel textured quads. A driver can see
// their backs or edges, so orient a second geometry around the same canopy.
// It shares the material and instance transforms and uses one draw per grove.
export function driverFoliage(overview) {
    const geometry = overview.clone()
    const vertices = geometry.getAttribute('position')
    const leafCenter = new Vector3()
    const corner = new Vector3()
    const face = new Vector3(0, 0, 1)
    const rotation = new Quaternion()
    for (let i = 0; i < vertices.count; i += 4) {
        leafCenter.fromBufferAttribute(vertices, i).add(corner.fromBufferAttribute(vertices, i + 3)).multiplyScalar(.5)
        rotation.setFromUnitVectors(face, corner.copy(leafCenter).normalize())
        for (let j = i; j < i + 4; j++) {
            corner.fromBufferAttribute(vertices, j).sub(leafCenter).applyQuaternion(rotation).add(leafCenter)
            vertices.setXYZ(j, corner.x, corner.y, corner.z)
        }
    }
    return geometry
}
