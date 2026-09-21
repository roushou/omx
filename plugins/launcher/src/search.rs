//! Deterministic local search over the platform's bounded catalogue.
//! Ranking lives in `omega::surface::Search`; this module adapts it to
//! candidates, launcher selection, and TypeSafe semantic ranking.

use crate::candidates::{Candidate, CandidateId};
use omega::platform::applications::ApplicationId;
use omega::surface::{Matchable, Search as SurfaceSearch};
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

impl Matchable for Candidate {
    fn id(&self) -> &str {
        match self.id() {
            CandidateId::Application(id) => id.as_str(),
            CandidateId::Action(id) => id.as_str(),
        }
    }

    fn name(&self) -> &str {
        self.name()
    }

    fn generic(&self) -> &str {
        self.generic_name()
    }

    fn keywords(&self) -> &[String] {
        self.keywords()
    }

    fn description(&self) -> &str {
        self.description()
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
        let ranked = SurfaceSearch::<Candidate>::new(query)
            .limit(limit)
            .find(entries, |candidate| candidate.is_favorite(favorites));
        Matches {
            items: ranked.items,
            total: ranked.total,
        }
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
}
