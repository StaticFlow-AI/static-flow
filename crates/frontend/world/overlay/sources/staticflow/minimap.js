import './minimap.css'

const storageKey = 'staticflow.world.minimap'
const zoom = 2.25
const clamp = (value, min, max) => Math.min(max, Math.max(min, value))
const percent = value => (value * 100).toFixed(2) + '%'
const labels = { altar: '世界线观测站', projects: '文章工坊', social: '广播会馆', landing: '未来道具实验室' }

export class WorldMinimap {
    constructor(map) {
        this.map = map
        this.game = map.game
        this.root = document.createElement('aside')
        this.root.id = 'world-minimap'
        this.root.setAttribute('aria-label', '小地图')
        this.root.innerHTML = `
            <button type="button" data-minimap-toggle aria-controls="world-minimap-view">
                <span>小地图</span><i aria-hidden="true"></i>
            </button>
            <button type="button" id="world-minimap-view" data-minimap-open aria-label="打开完整地图">
                <span class="minimap-field">
                    <img data-minimap-terrain alt="" draggable="false">
                    <span class="minimap-north" aria-hidden="true">N</span>
                    <i data-minimap-goal hidden></i>
                    <img data-minimap-player src="ui/map/player.webp" alt="" draggable="false">
                </span>
                <span class="minimap-caption"></span>
            </button>`
        this.game.domElement.append(this.root)
        this.toggle = this.root.querySelector('[data-minimap-toggle]')
        this.view = this.root.querySelector('[data-minimap-open]')
        this.terrain = this.root.querySelector('[data-minimap-terrain]')
        this.terrain.style.width = percent(zoom)
        this.player = this.root.querySelector('[data-minimap-player]')
        this.goal = this.root.querySelector('[data-minimap-goal]')
        this.caption = this.root.querySelector('.minimap-caption')

        let expanded = true
        try { expanded = localStorage.getItem(storageKey) !== 'hidden' } catch { /* Current visit only. */ }
        this.setExpanded(expanded)
        this.toggle.addEventListener('click', event => {
            this.setExpanded(this.view.hidden)
            if (event.detail) window.dispatchEvent(new Event('world-driving-focus'))
            try { localStorage.setItem(storageKey, this.view.hidden ? 'hidden' : 'visible') } catch { /* Current visit only. */ }
        })
        this.view.addEventListener('click', () => this.game.modals.open('map'))
        this.game.ticker.events.on('tick', () => {
            if (this.view.hidden || this.game.ticker.elapsed < this.nextUpdate) return
            this.nextUpdate = this.game.ticker.elapsed + .08
            if ((this.game.inputs.filters.has('wandering') || this.game.inputs.filters.has('racing')) &&
                !document.querySelector('dialog[open]')) this.update()
        }, 14)
    }

    setExpanded(expanded) {
        this.view.hidden = !expanded
        this.toggle.setAttribute('aria-expanded', String(expanded))
        this.toggle.setAttribute('aria-label', expanded ? '收起小地图' : '展开小地图')
        this.toggle.title = expanded ? '收起小地图' : '展开小地图'
        this.nextUpdate = 0
    }

    update() {
        const { game, map } = this
        const texture = map.textureUrl()
        if (texture !== this.texture) {
            this.texture = texture
            this.terrain.src = texture
        }

        const position = map.worldToMap(game.player.position)
        // Keep the player centered until the viewport meets the coast of the map.
        const origin = axis => clamp(axis - .5 / zoom, 0, 1 - 1 / zoom)
        const left = origin(position.x)
        const top = origin(position.y)
        this.terrain.style.transform = `translate(${percent(-left)}, ${percent(-top)})`
        const playerX = (position.x - left) * zoom
        const playerY = (position.y - top) * zoom
        this.player.style.left = percent(playerX)
        this.player.style.top = percent(playerY)
        this.player.style.transform = `translate(-50%, -50%) rotate(${(-game.physicalVehicle.yRotation).toFixed(3)}rad)`

        const { route, origin: observatory } = game.world.areas.altar
        const station = route.signal
        const showGoal = route.progress.active !== 'gate'
        this.goal.hidden = !showGoal
        if (this.station !== station || this.showGoal !== showGoal) {
            this.station = station
            this.showGoal = showGoal
            this.root.dataset.signal = route.target || 'observatory'
            this.caption.textContent = showGoal ? labels[station] : 'Steins Gate'
            this.view.setAttribute('aria-label', showGoal ? `打开完整地图，目的地：${labels[station]}` : '打开完整地图')
        }
        if (!showGoal) return
        const destination = map.worldToMap(route.stations[route.target]?.base || observatory)
        const x = (destination.x - left) * zoom
        const y = (destination.y - top) * zoom
        // At the coast the player is off-center. Start the bearing at the
        // player and stop it where it reaches the inset edge of the chart.
        const fromX = clamp(playerX, .06, .94)
        const fromY = clamp(playerY, .08, .92)
        const dx = x - fromX
        const dy = y - fromY
        const edgeX = dx < 0 ? .06 : .94
        const edgeY = dy < 0 ? .08 : .92
        const distance = Math.min(1, dx ? (edgeX - fromX) / dx : 1, dy ? (edgeY - fromY) / dy : 1)
        this.goal.style.left = percent(fromX + distance * dx)
        this.goal.style.top = percent(fromY + distance * dy)
        this.goal.classList.toggle('is-distant', distance < 1)
        if (distance < 1) this.goal.style.setProperty('--bearing', Math.atan2(dy, dx).toFixed(3) + 'rad')
    }
}
