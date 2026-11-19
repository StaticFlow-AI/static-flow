// Names with several thresholds keep their original progress counters.
const entries = {
    landingLeave: ['出发', '第一次驶出营地。'],
    areas: ['走遍世界', '到访所有区域。'],
    projects: ['博览群书', '翻遍文章工坊本次展出的文章。'],
    lab: ['唱片收藏家', '翻遍唱片实验室本次展出的唱片。'],
    whisper: ['路过留声', '留下一条路边留言。'],
    sea: ['潜水员', '开进海里看看。'],
    upsideDown: ['四脚朝天', '让小车翻过来。'],
    frontFlip: ['前空翻', '完成前空翻并四轮着地。'],
    backFlip: ['后空翻', '完成后空翻并四轮着地。'],
    suspensions: ['液压舞步', '试试小车的液压悬挂。'],
    honk: ['借过一下', '鸣笛十次。'],
    explosiveCrates: ['烟火大会', '引爆所有炸药箱。'],
    goHigh: ['腾空', '跃至十五米高。'],
    strike: ['全中', '一次击倒所有保龄球瓶。'],
    toiletDown: ['请勿打扰', '撞倒路边的厕所。'],
    circuitFinish: ['冲线', '跑完一场计时赛。'],
    circuitFinishFast: ['风驰电掣', '三十秒内完赛。'],
    circuitLeaderboard: ['榜上有名', '把圈速留在排行榜上。'],
    fullDay: ['从日出到日落', '经历一个完整的昼夜。'],
    sacrifice: ['世界线变动', '在观测站完成一次世界线校准。'],
    cataclysm: ['风暴之眼', '目睹一场巨变。'],
    weatherSnow: ['初雪', '在雪中开车。'],
    weatherRain: ['雨中漫游', '在雨中开车。'],
    lightning: ['晴天霹雳', '被闪电击中。'],
    waterfall: ['水幕后面', '看看瀑布背后有什么。'],
    fan: ['风扇出厂', '让一台电风扇落到地上。'],
    reset: ['物归原位', '将世界里的物件复位。'],
    statueDown: ['着陆偏差', '撞翻广播会馆屋顶的 FG204 时间机器。'],
    konami: ['上上下下', '输入那串经典游戏指令。'],
    debug: ['幕后视角', '打开调试界面。'],
    hacker: ['隐藏成就', '这枚徽章藏在哪里？'],
}

export function localizeAchievements(source) {
    return source.replace(/( {8}'(\w+)',\n) {8}'[^\n]*',\n {8}'[^\n]*',\n( {8}([^,\n]+))/g,
        (_, prefix, id, thresholdLine, threshold) => {
            let entry = entries[id]
            if (id === 'cookie') entry = ['饼干收集者', `收集 ${threshold} 块饼干。`]
            if (id === 'distanceDriven') entry = ['行车里程', `累计行驶 ${threshold} 公里。`]
            if (!entry) throw new Error('Missing achievement translation: ' + id)
            return prefix + '        ' + JSON.stringify(entry[0]) + ',\n        ' +
                JSON.stringify(entry[1]) + ',\n' + thresholdLine
        })
}
