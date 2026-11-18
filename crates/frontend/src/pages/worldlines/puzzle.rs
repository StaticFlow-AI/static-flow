//! Four observation gates form a chain. A D-Mail flips its gate and each
//! neighbor.

pub(super) const CONVERGED: u8 = 0b1111;
pub(super) const STATIONS: [(&str, &str); 4] =
    [("12:07", "收音机"), ("13:12", "站台"), ("15:26", "天台"), ("17:43", "邮筒")];

pub(super) struct Chapter {
    pub title: &'static str,
    pub incident: &'static str,
    pub ending: &'static str,
    initial: u8,
    letters: u8,
}

pub(super) const CHAPTERS: [Chapter; 3] = [
    Chapter {
        title: "雨夜的预报",
        incident: "收音机提前六小时播出了停电预告。你还有一次机会，让这段广播成为误报。",
        ending: "雨落下来。街灯亮着，收音机里只剩雨声。",
        initial: 0b1000,
        letters: 1,
    },
    Chapter {
        title: "迟到的末班车",
        incident: "站台时钟快了三分钟。寄信人赶上末班车的那段记忆，正在从记录里消失。",
        ending: "末班车开出站台。玻璃上，熟悉的身影朝你挥了挥手。",
        initial: 0b0010,
        letters: 2,
    },
    Chapter {
        title: "明天的回信",
        incident: "邮筒里躺着你明天写的信。信尾只有一句话：「这一次，把四个时刻都留下。」",
        ending: "你把信折好。这一次，信封上的日期终于是今天。",
        initial: 0b0111,
        letters: 3,
    },
];

pub(super) fn is_aligned(gates: u8, index: usize) -> bool {
    gates & (1 << index) != 0
}

/// Integer millionths preserve the meter's six digits across every route.
pub(super) fn divergence(gates: u8) -> String {
    let deviations = [124_000, 181_000, 92_000, 80_572];
    let difference: u32 = deviations
        .iter()
        .enumerate()
        .filter(|(index, _)| !is_aligned(gates, *index))
        .map(|(_, value)| value)
        .sum();
    let value = 1_048_596 - difference;
    format!("{}.{:06}", value / 1_000_000, value % 1_000_000)
}

#[derive(Clone, Copy, PartialEq)]
pub(super) enum Outcome {
    Playing,
    Delivered,
    Exhausted,
}

#[derive(Clone, Copy, PartialEq)]
pub(super) struct Run {
    pub chapter: usize,
    pub gates: u8,
    pub remaining: u8,
    pub last_mail: Option<usize>,
}

impl Run {
    pub fn start(chapter: usize) -> Self {
        let scenario = &CHAPTERS[chapter];
        Self {
            chapter,
            gates: scenario.initial,
            remaining: scenario.letters,
            last_mail: None,
        }
    }

    pub fn outcome(self) -> Outcome {
        match (self.gates, self.remaining) {
            (CONVERGED, _) => Outcome::Delivered,
            (_, 0) => Outcome::Exhausted,
            _ => Outcome::Playing,
        }
    }

    pub fn send(&mut self, station: usize) {
        if station < STATIONS.len() && self.outcome() == Outcome::Playing {
            let gate = 1 << station;
            self.gates ^= (gate | gate << 1 | gate >> 1) & CONVERGED;
            self.remaining -= 1;
            self.last_mail = Some(station);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_chapter_has_one_winning_set_within_its_mail_budget() {
        for chapter in 0..CHAPTERS.len() {
            let start = Run::start(chapter);
            let winners: Vec<_> = (0u8..=CONVERGED)
                .filter(|choices| choices.count_ones() <= u32::from(start.remaining))
                .filter(|choices| {
                    let mut run = start;
                    for station in 0..STATIONS.len() {
                        if is_aligned(*choices, station) {
                            run.send(station);
                        }
                    }
                    run.outcome() == Outcome::Delivered
                })
                .collect();
            assert_eq!(winners.len(), 1);
            assert_eq!(winners[0].count_ones(), u32::from(start.remaining));
        }
    }

    #[test]
    fn terminal_runs_ignore_further_letters() {
        for (station, expected) in [(0, Outcome::Exhausted), (1, Outcome::Delivered)] {
            let mut run = Run::start(0);
            run.send(station);
            assert!(run.outcome() == expected);
            let gates = run.gates;
            run.send(2);
            assert_eq!((run.gates, run.remaining), (gates, 0));
        }
    }
}
