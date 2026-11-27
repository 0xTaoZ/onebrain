//! Who answers a chat message that mentions nobody.
//!
//! Decided without a model call, in this order: the only member; the agent the
//! user is replying to; the member whose name and role best match the words of
//! the message (terms every member shares count for nothing, idle members win
//! ties); otherwise nobody is picked and the caller falls back to the lead.

use std::collections::{HashMap, HashSet};

use uuid::Uuid;

/// `@所有人`, `@大家`, `@all` or `@everyone`: ask every member once.
pub(super) fn is_everyone_handle(handle: &str) -> bool {
    let lower = handle.trim_start_matches('@').to_lowercase();
    matches!(lower.as_str(), "all" | "everyone" | "所有人" | "大家" | "全体")
}

pub(super) struct RouteCandidate<'a> {
    pub id: Uuid,
    pub name: &'a str,
    pub role: &'a str,
    pub busy: bool,
}

fn is_cjk(c: char) -> bool {
    matches!(c as u32, 0x3040..=0x30FF | 0x3400..=0x4DBF | 0x4E00..=0x9FFF | 0xAC00..=0xD7AF)
}

/// Lower-cased Latin words of three or more letters, plus CJK character pairs.
fn terms(text: &str) -> HashSet<String> {
    let mut out = HashSet::new();
    let mut word = String::new();
    let mut cjk: Vec<char> = Vec::new();
    let flush = |word: &mut String, cjk: &mut Vec<char>, out: &mut HashSet<String>| {
        if word.chars().count() >= 3 {
            out.insert(word.to_lowercase());
        }
        word.clear();
        for pair in cjk.windows(2) {
            out.insert(pair.iter().collect());
        }
        cjk.clear();
    };
    for c in text.chars() {
        if is_cjk(c) {
            if !word.is_empty() {
                let mut held = std::mem::take(&mut cjk);
                flush(&mut word, &mut Vec::new(), &mut out);
                cjk.append(&mut held);
            }
            cjk.push(c);
        } else if c.is_alphanumeric() {
            if !cjk.is_empty() {
                flush(&mut String::new(), &mut cjk, &mut out);
            }
            word.push(c);
        } else {
            flush(&mut word, &mut cjk, &mut out);
        }
    }
    flush(&mut word, &mut cjk, &mut out);
    out
}

pub(super) fn pick_member(
    text: &str,
    candidates: &[RouteCandidate<'_>],
    replying_to: Option<Uuid>,
) -> Option<Uuid> {
    match candidates {
        [] => return None,
        [only] => return Some(only.id),
        _ => {}
    }
    if let Some(id) = replying_to.filter(|id| candidates.iter().any(|c| c.id == *id)) {
        return Some(id);
    }
    let wanted = terms(text);
    if wanted.is_empty() {
        return None;
    }
    let profiles: Vec<HashSet<String>> = candidates
        .iter()
        .map(|c| terms(&format!("{} {}", c.name, c.role)))
        .collect();
    // A term found in every profile does not tell members apart.
    let mut seen_in: HashMap<&str, usize> = HashMap::new();
    for profile in &profiles {
        for term in profile {
            *seen_in.entry(term.as_str()).or_default() += 1;
        }
    }
    let total = candidates.len() as f64;
    let mut best: Option<(f64, bool, Uuid)> = None;
    for (candidate, profile) in candidates.iter().zip(&profiles) {
        let score: f64 = wanted
            .iter()
            .filter(|term| profile.contains(*term))
            .map(|term| (total / seen_in[term.as_str()] as f64).ln())
            .sum();
        if score <= 0.0 {
            continue;
        }
        let better = match best {
            None => true,
            Some((best_score, best_busy, _)) => {
                score > best_score + 1e-9 || ((score - best_score).abs() < 1e-9 && best_busy && !candidate.busy)
            }
        };
        if better {
            best = Some((score, candidate.busy, candidate.id));
        }
    }
    best.map(|(_, _, id)| id)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids() -> (Uuid, Uuid, Uuid) {
        (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4())
    }

    #[test]
    fn picks_by_role_words_in_english_and_chinese() {
        let (dev, review, docs) = ids();
        let members = [
            RouteCandidate { id: dev, name: "claude", role: "Builds features and fixes bugs. 负责写代码和修复问题", busy: false },
            RouteCandidate { id: review, name: "codex", role: "Reviews pull requests and checks security. 负责代码审查", busy: false },
            RouteCandidate { id: docs, name: "gemini", role: "Writes documentation and README. 负责写文档", busy: false },
        ];
        assert_eq!(pick_member("please review the security of this change", &members, None), Some(review));
        assert_eq!(pick_member("帮我更新一下文档", &members, None), Some(docs));
        assert_eq!(pick_member("fix the login bugs", &members, None), Some(dev));
        assert_eq!(pick_member("hello there", &members, None), None, "no signal: fall back to the lead");
    }

    #[test]
    fn everyone_handles_are_recognised() {
        for handle in ["all", "@everyone", "所有人", "大家", "ALL"] {
            assert!(is_everyone_handle(handle), "{handle}");
        }
        assert!(!is_everyone_handle("allison"));
    }

    #[test]
    fn single_member_and_replies_win_and_idle_breaks_ties() {
        let (a, b, _) = ids();
        let one = [RouteCandidate { id: a, name: "claude", role: "", busy: true }];
        assert_eq!(pick_member("anything", &one, None), Some(a));

        let two = [
            RouteCandidate { id: a, name: "claude", role: "tests", busy: true },
            RouteCandidate { id: b, name: "codex", role: "tests", busy: false },
        ];
        assert_eq!(pick_member("run the tests", &two, Some(a)), Some(a), "a reply goes to the agent replied to");
        let tie = [
            RouteCandidate { id: a, name: "claude", role: "frontend", busy: true },
            RouteCandidate { id: b, name: "codex", role: "frontend", busy: false },
        ];
        assert_eq!(pick_member("frontend work", &tie, None), None, "shared words do not decide");
    }
}
