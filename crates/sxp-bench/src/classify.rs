//! Table-driven SM-1 verdict. Exploratory runs do not receive a scientific label.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Label {
    Positive,
    Marginal,
    Negative,
    Inconclusive,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClassScore {
    pub class: String,
    pub evaluation_inputs: u64,
    pub wins: u64,
    pub full_bytes: u64,
    pub baselines_only_bytes: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Policy {
    pub eligible: u64,
    pub winning: u64,
    pub clearing: u64,
    pub growth_percent: u64,
    pub gain_denominator: &'static str,
    pub exploratory: bool,
    pub label: Option<Label>,
    pub opens_pcm_experiments: bool,
}

/// Classify measured real classes.
///
/// `reportable` false returns exploratory and no scientific label. A class wins
/// when its Baselines-only sum is strictly larger than full MBSC. The growth
/// bar is `gain * 100 >= baselines_only_bytes` and `wins * 2 > evaluation_inputs`.
pub fn classify(reportable: bool, classes: &[ClassScore]) -> Policy {
    let mut eligible = 0u64;
    let mut winning = 0u64;
    let mut clearing = 0u64;
    for class in classes {
        if !is_real(&class.class) || class.evaluation_inputs == 0 {
            continue;
        }
        eligible = eligible.saturating_add(1);
        let gain = class
            .baselines_only_bytes
            .saturating_sub(class.full_bytes.min(class.baselines_only_bytes));
        let won = class.baselines_only_bytes > class.full_bytes;
        if !won {
            continue;
        }
        winning = winning.saturating_add(1);
        let percent = gain.saturating_mul(100) >= class.baselines_only_bytes;
        let majority = class.wins.saturating_mul(2) > class.evaluation_inputs;
        if percent && majority {
            clearing = clearing.saturating_add(1);
        }
    }
    if !reportable {
        return Policy {
            eligible,
            winning,
            clearing,
            growth_percent: 1,
            gain_denominator: "baselines-only class sum",
            exploratory: true,
            label: None,
            opens_pcm_experiments: false,
        };
    }
    let (label, opens) = if eligible < 2 {
        (Label::Inconclusive, false)
    } else if winning == 0 {
        (Label::Negative, false)
    } else if winning == 1 {
        (Label::Negative, true)
    } else if clearing < 2 {
        (Label::Marginal, false)
    } else {
        (Label::Positive, false)
    };
    Policy {
        eligible,
        winning,
        clearing,
        growth_percent: 1,
        gain_denominator: "baselines-only class sum",
        exploratory: false,
        label: Some(label),
        opens_pcm_experiments: opens,
    }
}

fn is_real(class: &str) -> bool {
    matches!(class, "text" | "pcm" | "executable")
}
