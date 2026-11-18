use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
pub enum Line {
    #[default]
    Alpha,
    Echo,
    Beta,
    Gate,
}
impl Line {
    pub fn id(self) -> &'static str {
        match self {
            Self::Alpha => "alpha",
            Self::Echo => "echo",
            Self::Beta => "beta",
            Self::Gate => "gate",
        }
    }
    pub fn number(self) -> &'static str {
        match self {
            Self::Alpha => "0.571024",
            Self::Echo => "0.337187",
            Self::Beta => "1.129840",
            Self::Gate => "1.048596",
        }
    }
    pub fn time(self) -> &'static str {
        match self {
            Self::Alpha => "17:43",
            Self::Echo => "17:56",
            Self::Beta => "17:52",
            Self::Gate => "17:44",
        }
    }
}
pub const MEMBERS: [(&str, &str); 8] = [
    ("okabe", "冈部伦太郎"),
    ("mayuri", "椎名真由理"),
    ("daru", "桥田至"),
    ("kurisu", "牧濑红莉栖"),
    ("moeka", "桐生萌郁"),
    ("luka", "漆原琉华"),
    ("faris", "菲莉丝"),
    ("suzuha", "阿万音铃羽"),
];
pub fn name(id: &str) -> &'static str {
    match id {
        "phone" => "茶几上的手机",
        "crt" => "42 英寸显像管",
        "wave" => "电话微波炉（暂定）",
        "leap" => "时间跳跃耳机",
        "upa" => "沙发边的金属乌帕",
        _ => MEMBERS
            .iter()
            .find(|(key, _)| *key == id)
            .map_or("", |(_, s)| *s),
    }
}
pub struct Dmail {
    pub to: &'static str,
    pub body: &'static str,
}
pub const CHOICES: [Dmail; 3] = [
    Dmail {
        to: "椎名真由理",
        body: "东口等我。我来接你。",
    },
    Dmail {
        to: "菲莉丝 · 秋叶原商店街",
        body: "请通知站务，关闭东口。",
    },
    Dmail {
        to: "椎名真由理、桥田至",
        body: "带乌帕回研究所。两个人一起。",
    },
];

#[derive(Clone, PartialEq, Eq)]
pub enum Panel {
    Phone,
    Talk(&'static str, &'static str),
    Leap,
    Journal,
    Help,
    Restart,
}
#[derive(Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct Story {
    pub line: Line,
    pub phone: bool,
    pub crt: bool,
    pub armed: bool,
    pub upa: bool,
    pub roof_tip: bool,
    pub leaps: u32,
    #[serde(skip)]
    pub epoch: u32,
    pub witness: bool,
    pub reunion: bool,
    pub mail: Vec<usize>,
    #[serde(skip)]
    pub panel: Option<Panel>,
}
impl Story {
    pub fn objective(&self) -> (&'static str, &'static str) {
        match self.line {
            Line::Echo if !self.witness => {
                ("daru", "真由理没有回来。问电脑桌前的桥田至，最后一通电话说了什么。")
            },
            Line::Beta if !self.witness => {
                ("mayuri", "桥田的椅子空了。问沙发旁的真由理，她在车站看见了谁。")
            },
            Line::Echo | Line::Beta => ("leap", "在电脑桌左端戴上耳机，把这一段记忆带回 17:43。"),
            Line::Gate if self.reunion => {
                ("", "两个人都回来了。你可以留在研究所，听大家聊聊那个下午。")
            },
            Line::Gate => ("mayuri", "沙发那头有人在等你。去和真由理一起拆开扭蛋。"),
            Line::Alpha if !self.phone => ("phone", "沙发前的茶几上，一部手机正在振动。走近调查。"),
            Line::Alpha if self.leaps > 0 && !self.upa => {
                ("upa", "你记得银色的反光。在沙发的熊猫靠垫旁找金属乌帕。")
            },
            Line::Alpha if self.leaps > 0 && !self.roof_tip => {
                ("suzuha", "带乌帕去屋顶找铃羽。出二楼东侧的门，走外侧向上的楼梯。")
            },
            Line::Alpha if !self.crt => {
                ("crt", "下楼接通布朗管工房的 42 英寸电视。出二楼东侧的门，走靠墙向下的楼梯。")
            },
            Line::Alpha if !self.armed => {
                ("wave", "回到二楼，在装着青香蕉的电话微波炉上设定逆转。")
            },
            Line::Alpha => ("wave", "微波炉已就绪。打开随身手机，选择一封 D-Mail 发给过去。"),
        }
    }
    pub fn incoming(&self) -> (&'static str, &'static str) {
        match self.line {
            Line::Alpha if self.leaps > 0 => (
                "草稿箱 · 17:43",
                "草稿是空的，你却记得东口的广播。给过去的人一个足够让他们留下的理由。",
            ),
            Line::Alpha => (
                "未知号码 · 明日 17:56",
                "如果你还能收到，别让真由理一个人到秋叶原站东口。她怀表的秒针会停。",
            ),
            Line::Echo => {
                ("桥田至 · 17:56", "我到了东口，只看到她落下的怀表。你真的把那条短信发出去了吗？")
            },
            Line::Beta => {
                ("真由理 · 17:52", "我在研究所。阿至折回去帮最后一个乘客了，他的手机一直打不通。")
            },
            Line::Gate => (
                "研究所群组 · 17:44",
                "真由氏抽到了银色乌帕！阿至说要拿显像管的纸箱给它做个家。晚饭等你。",
            ),
        }
    }
    fn say(&mut self, who: &'static str, words: &'static str) {
        self.panel = Some(Panel::Talk(who, words));
    }
    pub fn inspect(&mut self, id: &str) {
        match id {
            "phone" => {
                self.phone = true;
                self.panel = Some(Panel::Phone);
            },
            "crt" if self.line == Line::Alpha => {
                self.crt = true;
                self.say(
                    "42 英寸显像管",
                    "开关咔哒一声。绿光从屏幕中央铺开，楼板上传来微波炉应答的短鸣。保持电视开启，\
                     回二楼设定逆转。",
                );
            },
            "wave" if self.line != Line::Alpha => self.say(
                "电话微波炉（暂定）",
                "香蕉还在转盘上，定时器却已经归零。时间跳跃机能把记得这一切的你送回发信之前。",
            ),
            "wave" if !self.phone => self.say(
                "电话微波炉（暂定）",
                "门里是三根青香蕉。一截改装的手机线垂到桌沿，茶几上有手机在响。",
            ),
            "wave" if !self.crt => self.say(
                "电话微波炉（暂定）",
                "短鸣之后，定时器熄灭。接收端没有电。红莉栖指了指楼下那台 42 英寸显像管。",
            ),
            "wave" if self.armed => self.panel = Some(Panel::Phone),
            "wave" => {
                self.armed = true;
                self.say(
                    "电话微波炉（暂定）",
                    "你合上炉门，把定时器拨向逆转。转盘开始旋动，\
                     手机屏幕上多出了「投递至昨天」的选项。",
                );
            },
            "leap" if matches!(self.line, Line::Echo | Line::Beta) && self.witness => {
                self.panel = Some(Panel::Leap)
            },
            "leap" => self.say(
                "时间跳跃机",
                "耳罩上贴着红莉栖的字条。「只能送走记忆。先弄清楚你究竟想记住什么。」",
            ),
            "upa" => {
                self.upa = true;
                self.say(
                    "金属乌帕",
                    "熊猫靠垫的接缝里藏着一枚银色扭蛋。外壳上贴着「两人份」的小纸条，\
                     背面是铃羽画的自行车。",
                );
            },
            "suzuha" if self.upa => {
                self.roof_tip = true;
                self.say(
                    name(id),
                    "这是我留给他们的暗号。昨天说好了，如果抽中这枚乌帕，\
                     两个人就一起在研究所拆蛋。试着提醒他们那个约定。",
                );
            },
            "daru" if self.line == Line::Echo => {
                self.witness = true;
                self.say(
                    name(id),
                    "我赶到了，她却还站在东口，因为她答应过你要等。广播响的时候，\
                     我听见听筒里有金属扭蛋滚落的声音。记住这个。",
                );
            },
            "mayuri" if self.line == Line::Beta => {
                self.witness = true;
                self.say(
                    name(id),
                    "阿至说还有人没出来。真由氏想，要是我们昨天约好一起拆那枚乌帕，\
                     今天他就不会在站台上了吧。",
                );
            },
            "mayuri" if self.line == Line::Gate => {
                self.reunion = true;
                self.say(
                    name(id),
                    "听到了吗？我的怀表又在走了。阿至，把手伸过来呀，\
                     是两个人一起才可以打开的扭蛋。",
                );
            },
            "okabe" => self.say(
                name(id),
                "这间屋子每经历一次重写，就少一个人记得它原来的样子。你如果还记得，\
                 就替我把他们的名字喊出来。",
            ),
            "mayuri" => self.say(
                name(id),
                "嘟嘟噜。真由氏的怀表慢了一格，东口的钟明明比它更慢呢。\
                 我的银色小家伙跑到哪里去啦？",
            ),
            "daru" => self.say(
                name(id),
                "电话线、微波炉、楼下那台显像管。少一台在线，我的杰作就只能负责加热便当。\
                 耳机放在我桌子的左边。",
            ),
            "kurisu" => self.say(
                name(id),
                "D-Mail 改变过去收到的信息。时间跳跃只改变你记得什么。救人之前，\
                 想清楚你要让谁做出不同的选择。",
            ),
            "moeka" => self.say(
                name(id),
                "她举起手机给你看一行字。「东口的监控只拍到了出口。要找没回来的人，\
                 先问回来了的人。」",
            ),
            "luka" => self.say(
                name(id),
                "昨天真由理和桥田先生来借过一个小纸袋。她说，普通的礼物也可以有很重要的约定。",
            ),
            "faris" => self.say(
                name(id),
                "车站的人流跟秋叶原的猫一样，堵住一条巷子就会换一条。真正让人停下来的，\
                 是有人在家里等他喵。",
            ),
            "suzuha" => self.say(
                name(id),
                "站在这里能看见东口。昨天我留了一张纸条在二楼沙发上，\
                 夹在一个比普通扭蛋更沉的小家伙身上。",
            ),
            "crt" => self.say(name(id), "屏幕留着一条细细的余辉。这台旧电视已经照见过另一个下午。"),
            _ => {},
        }
    }
    pub fn can_send(&self, i: usize) -> bool {
        self.phone
            && self.crt
            && self.armed
            && self.line == Line::Alpha
            && (i < 2 || (i == 2 && self.leaps > 0 && self.roof_tip && self.upa))
    }
    pub fn send(&mut self, i: usize) {
        if !self.can_send(i) {
            return;
        }
        self.mail.push(i);
        self.line = [Line::Echo, Line::Beta, Line::Gate][i];
        self.armed = false;
        self.witness = false;
        self.epoch += 1;
        self.panel = None;
    }
    pub fn leap(&mut self) {
        if !self.witness || !matches!(self.line, Line::Echo | Line::Beta) {
            return;
        }
        self.line = Line::Alpha;
        self.crt = false;
        self.armed = false;
        self.witness = false;
        self.leaps += 1;
        self.epoch += 1;
        self.panel = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn both_failed_rescues_give_a_reachable_return_and_joint_rescue() {
        for (choice, survivor) in [(0, "daru"), (1, "mayuri")] {
            let mut s = Story::default();
            s.inspect("phone");
            s.inspect("wave");
            s.send(choice);
            assert_eq!(s.epoch, 0);
            s.inspect("crt");
            s.inspect("wave");
            s.send(2);
            assert_eq!(s.epoch, 0);
            s.send(choice);
            s.leap();
            assert_eq!(s.leaps, 0);
            s.inspect(survivor);
            s.inspect("leap");
            assert!(matches!(s.panel, Some(Panel::Leap)));
            s.leap();
            assert_eq!(s.leaps, 1);
            assert!(!s.crt && !s.armed);
            assert_eq!(s.mail, [choice]);
            s.inspect("upa");
            s.inspect("suzuha");
            s.inspect("crt");
            s.inspect("wave");
            s.send(2);
            assert!(s.line == Line::Gate);
            s.inspect("mayuri");
            assert!(s.reunion);
        }
    }
}
