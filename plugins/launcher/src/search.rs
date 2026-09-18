//! Deterministic local search over the platform's bounded catalogue.

use omega::platform::applications::{Application, ApplicationId};
use std::collections::BTreeSet;

pub(crate) struct Matches<'a> {
    pub(crate) items: Vec<&'a Application>,
    pub(crate) total: usize,
}

impl Matches<'_> {
    pub(crate) fn selected(&self, selected: Option<&ApplicationId>) -> Option<&Application> {
        self.items
            .iter()
            .find(|app| Some(app.id()) == selected)
            .copied()
            .or_else(|| self.items.first().copied())
    }

    pub(crate) fn contains(&self, id: &ApplicationId) -> bool {
        self.items.iter().any(|app| app.id() == id)
    }
}

pub(crate) struct Search;

impl Search {
    pub(crate) fn find<'a>(
        entries: &'a [Application],
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
            let id = app.id().as_str().to_lowercase();
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

                let personal = query.is_empty() && !favorites.contains(app.id());
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
