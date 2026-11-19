// The game is a standalone document. Use the same site routes as the Yew shell.
const site = location.pathname.startsWith('/static_flow/') ? '/static_flow' : ''
export const sitePath = path => site + path
const apiBase = (import.meta.env.VITE_STATICFLOW_API_BASE || '/api').replace(/\/$/, '')
const itemPath = (collection, id) => sitePath(collection + '/' + encodeURIComponent(id))
const short = (value, length) => Array.from(value).length <= length ? value : Array.from(value).slice(0, length - 1).join('') + '…'

export const categories = []
export const articles = []
export const records = []
export const playlist = []

export const destinations = [
    ['最新文章', '/latest'], ['音乐库', '/media/audio'], ['图片库', '/media/image'],
    ['读者工坊', '/studio'], ['文章索引', '/posts'], ['分类', '/categories'],
    ['标签', '/tags'], ['全文搜索', '/search'],
].map(([name, path], index) => ({ name, url: sitePath(path), align: index < 6 ? 'right' : 'left' }))

// Keep failures local to the relevant building. A world visit never depends on a
// specific article, song or the content store being available.
async function list(path, field) {
    try {
        const response = await fetch(apiBase + path, { signal: AbortSignal.timeout(6000) })
        if (response.ok) {
            const payload = await response.json()
            if (Array.isArray(payload[field])) return payload[field]
        }
    } catch {}
    return []
}

function project(row) {
    const url = row ? itemPath('/posts', row.id) : sitePath('/posts')
    const name = row?.title || '文章索引'
    const section = row?.category || 'StaticFlow'
    const coverId = /^images\/([a-f\d]{64})$/.exec(row?.featured_image || '')?.[1]
    const card = { name, section, byline: [row?.author, row?.date].filter(Boolean).join('   /   '),
        art: coverId ? apiBase + '/images/' + coverId + '?thumb=true' : null }
    const images = [card]
    if (row?.summary) images.push({ ...card, art: null, excerpt: row.summary })
    return {
        title: short(name, 12), fullTitle: name, titleSmall: [short(name, 7)],
        url, attributes: { role: row?.author, at: short(section, 10), with: row?.date },
        distinctions: [], images,
    }
}

function record(row) {
    const name = row?.title || '音乐库'
    const card = { name, section: 'STATICFLOW RECORDS', byline: row?.artist || '音乐库', vinyl: true }
    return {
        title: short(name, 12), url: row ? itemPath('/media/audio', row.id) : sitePath('/media/audio'),
        image: card, imageMini: card,
    }
}

function mountLinks(selector, links) {
    for (const element of document.querySelectorAll(selector)) {
        const list = document.createElement('ul')
        for (const [title, url] of links) {
            const link = document.createElement('a')
            link.textContent = title
            link.href = url
            const item = document.createElement('li')
            item.append(link)
            list.append(item)
        }
        element.replaceChildren(list)
    }
}

async function mountGallery(element) {
    const gallery = await list('/images/random?limit=6', 'images')
    for (const row of gallery) {
        if (!row || typeof row.id !== 'string' || !row.id) continue
        const image = new Image()
        image.loading = 'lazy'
        image.decoding = 'async'
        image.alt = row.filename && !/^[a-f\d]{64}/.test(row.filename) ? row.filename : '博客图库作品'
        image.src = apiBase + '/images/' + encodeURIComponent(row.id) + '?thumb=true'
        element.append(image)
    }
    const link = document.createElement('a')
    link.href = sitePath('/media/image')
    link.textContent = '浏览完整图库 ↗'
    element.append(link)
}

export async function loadBlog() {
    const [posts, music, groups] = await Promise.all([
        list('/articles?limit=8', 'articles'), list('/music?limit=8', 'songs'), list('/categories', 'categories'),
    ])
    const valid = rows => rows.filter(row => typeof row?.id === 'string' && typeof row.title === 'string')
    articles.push(...valid(posts).map(project))
    records.push(...valid(music).map(record))
    for (const song of valid(music)) {
        if (['mp3', 'flac', 'aac', 'ogg', 'wav', 'm4a'].includes(song.format)) {
            playlist.push({ path: apiBase + '/music/' + encodeURIComponent(song.id) + '/audio',
                format: song.format, name: [song.title, song.artist].filter(Boolean).join(' · ') })
        }
    }
    categories.push(...groups.filter(row => typeof row?.name === 'string' && Number.isInteger(row.count))
        .sort((a, b) => b.count - a.count).map(group => ({ ...group, url: itemPath('/categories', group.name) })))
    if (!articles.length) articles.push(project(null))
    if (!records.length) records.push(record(null))
    if (!categories.length) categories.push({ name: '分类索引', count: 0, url: sitePath('/categories') })

    const gallery = document.querySelector('[data-sf-gallery]')
    if (gallery) {
        const observer = new IntersectionObserver(entries => {
            if (entries.some(entry => entry.isIntersecting)) {
                observer.disconnect()
                void mountGallery(gallery)
            }
        })
        observer.observe(gallery)
    }

    mountLinks('[data-sf-articles]', articles.map(item => [item.fullTitle, item.url]))
    mountLinks('[data-sf-categories]', categories.slice(0, 6).map(item => [item.name, item.url]))
}
