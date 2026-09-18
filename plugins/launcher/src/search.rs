//! Deterministic local search over the platform's bounded catalogue.

use crate::candidates::{Candidate, CandidateId};
use omega::platform::applications::ApplicationId;
use std::collections::BTreeSet;

pub(crate) struct Matches<'a> {
    pub(crate) items: Vec<&'a Candidate>,
    pub(crate) total: usize,
}

impl Matches<'_> {
    pub(crate) fn selected(&self, selected: Option<&CandidateId>) -> Option<&Candidate> {
        self.items
            .iter()
            .find(|app| Some(app.id()) == selected)
            .copied()
            .or_else(|| self.items.first().copied())
    }

    pub(crate) fn contains(&self, id: &CandidateId) -> bool {
        self.items.iter().any(|app| app.id() == id)
    }
}

pub(crate) struct Search;

impl Search {
    pub(crate) fn find<'a>(
        entries: &'a [Candidate],
        query: &str,
        limit: u8,
        favorites: &BTreeSet<ApplicationId>,
    ) -> Matches<'a> {
        let query = query.trim().to_lowercase();
        let tokens: Vec<_> = query.split_whitespace().collect();
        let mut matches = Vec::new();

        for app in entries {
            let name = app.name().to_lowercase();
            let generic = app.generic_name().to_lowercase();
            let keywords = app.keywords().join(" ").to_lowercase();
            let description = app.description().to_lowercase();
            let id = app.id().to_string().to_lowercase();
            let mut score = 0usize;
            let mut matched = true;

            for token in &tokens {
                let rank = if name.starts_with(token) {
                    0
                } else if name.contains(token) {
                    1
                } else if generic.contains(token) || keywords.contains(token) {
                    2
                } else if description.contains(token) {
                    3
                } else if id.contains(token) {
                    4
                } else if let Some(cost) = Self::fuzzy(token, &name) {
                    10 + cost
                } else {
                    matched = false;
                    break;
                };
                score += rank;
            }

            if matched {
                let priority = if query.is_empty() || name == query {
                    0
                } else if name.starts_with(&query) {
                    1
                } else {
                    2
                };

                let personal = query.is_empty() && !app.is_favorite(favorites);
                matches.push((priority, score, personal, name, app));
            }
        }

        matches
            .sort_by(|a, b| (a.0, a.1, a.2, &a.3, a.4.id()).cmp(&(b.0, b.1, b.2, &b.3, b.4.id())));
        let total = matches.len();
        let items = matches
            .into_iter()
            .take(usize::from(limit.clamp(1, 100)))
            .map(|(_, _, _, _, app)| app)
            .collect();
        Matches { items, total }
    }

    pub(crate) fn semantic<'a>(
        entries: &'a [Candidate],
        scores: &std::collections::BTreeMap<String, f64>,
        limit: u8,
    ) -> Matches<'a> {
        let mut ranked = Vec::new();
        for entry in entries {
            if let Some(score) = scores.get(&entry.id().to_string())
                && *score >= 1.0
            {
                ranked.push((*score, entry));
            }
        }
        ranked.sort_by(|a, b| {
            b.0.total_cmp(&a.0)
                .then_with(|| a.1.name().cmp(b.1.name()))
                .then_with(|| a.1.id().cmp(b.1.id()))
        });
        let total = ranked.len();
        Matches {
            items: ranked
                .into_iter()
                .take(usize::from(limit.clamp(1, 100)))
                .map(|(_, entry)| entry)
                .collect(),
            total,
        }
    }

    fn fuzzy(token: &str, name: &str) -> Option<usize> {
        let mut wanted = token.chars();
        let mut next = wanted.next()?;
        let mut first = None;
        let mut matched = 0;
        for (index, character) in name.chars().enumerate() {
            if character != next {
                continue;
            }
            first.get_or_insert(index);
            matched += 1;
            match wanted.next() {
                Some(character) => next = character,
                None => return Some(index + 1 - matched + first.unwrap_or(0)),
            }
        }
        None
    }
}
