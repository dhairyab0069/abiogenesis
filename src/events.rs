//! A record of how every chain came to exist: generated from nothing, mutated
//! from one parent, or recombined from two.
//!
//! Design choices:
//! - **Every chain gets an id, and every change makes a new one.** A mutation
//!   still replaces its parent in the soup (so the population stays stable), but
//!   in the record the child is a new chain with a link back to its parent. That
//!   turns the history into a graph we can draw as a lineage tree.
//! - **Events say exactly what changed** (which residue, which cut points), not
//!   just "something mutated". The build-up view needs that to flash the right tile.
//! - **The log is append-only and tagged with the step.** Views keep a cursor and
//!   read only what is new, so a view never has to diff two populations.
//! - **No simulation code in here.** It is plain data, so it is easy to test and
//!   easy to reuse in stage 5's time-lapse.

/// Identifies one chain for its whole life. Never reused.
pub type ChainId = u64;

/// The exact edit a mutation made. `index` counts residues from the start of the parent.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mutation {
    Substitution { index: usize, from: char, to: char },
    Insertion { index: usize, residue: char },
    Deletion { index: usize, residue: char },
}

/// One thing that happened to the chains.
#[derive(Clone, Debug, PartialEq)]
pub enum ChainEvent {
    /// A brand new random chain appeared.
    Generated { id: ChainId, sequence: String },
    /// `parent` mutated into `id`.
    Mutated { id: ChainId, parent: ChainId, mutation: Mutation, sequence: String },
    /// `id` is the start of `parents.0` (up to `cut.0`) joined to the end of
    /// `parents.1` (from `cut.1`).
    Recombined { id: ChainId, parents: (ChainId, ChainId), cut: (usize, usize), sequence: String },
    /// A protein, read off an RNA strand: `gene` is the strand's bases, `sequence`
    /// the amino acids they spell. The strand is not a chain, so there is no parent.
    Translated { id: ChainId, gene: String, sequence: String },
}

impl ChainEvent {
    /// The chain this event created.
    pub fn id(&self) -> ChainId {
        match self {
            Self::Generated { id, .. }
            | Self::Mutated { id, .. }
            | Self::Recombined { id, .. }
            | Self::Translated { id, .. } => *id,
        }
    }

    /// The new chain's sequence.
    pub fn sequence(&self) -> &str {
        match self {
            Self::Generated { sequence, .. }
            | Self::Mutated { sequence, .. }
            | Self::Recombined { sequence, .. }
            | Self::Translated { sequence, .. } => sequence,
        }
    }

    /// A one-word description of how the chain came to be.
    pub fn how(&self) -> &'static str {
        match self {
            Self::Generated { .. } => "generated",
            Self::Mutated { .. } => "mutated",
            Self::Recombined { .. } => "recombined",
            Self::Translated { .. } => "translated",
        }
    }

    /// The chains this one came from: none, one or two.
    pub fn parents(&self) -> Vec<ChainId> {
        match self {
            Self::Generated { .. } | Self::Translated { .. } => vec![],
            Self::Mutated { parent, .. } => vec![*parent],
            Self::Recombined { parents, .. } => vec![parents.0, parents.1],
        }
    }
}

/// One line of a lineage tree.
#[derive(Clone, Debug, PartialEq)]
pub struct LineageRow {
    pub id: ChainId,
    /// 0 for the root, 1 for its children, and so on.
    pub depth: usize,
    pub step: u64,
    /// How it came to be: "translated", "mutated" or "recombined".
    pub how: &'static str,
    pub sequence: String,
}

/// An event and the step it happened on.
#[derive(Clone, Debug, PartialEq)]
pub struct Stamped {
    pub step: u64,
    pub event: ChainEvent,
}

/// The id that means "no recorded chain": small molecules, and chains added to the
/// soup without going through the log. Real ids start at 1, so a parent link can
/// never silently point at the first real chain by mistake.
pub const UNRECORDED: ChainId = 0;

/// The append-only history. Also hands out fresh ids.
#[derive(Clone, Debug)]
pub struct EventLog {
    events: Vec<Stamped>,
    next_id: ChainId,
}

impl Default for EventLog {
    fn default() -> Self {
        Self { events: Vec::new(), next_id: UNRECORDED + 1 }
    }
}

impl EventLog {
    pub fn new() -> Self {
        Self::default()
    }

    /// A fresh id. Call this before building the event that uses it.
    pub fn new_id(&mut self) -> ChainId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn record(&mut self, step: u64, event: ChainEvent) {
        self.events.push(Stamped { step, event });
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Everything recorded after the first `cursor` events. A view stores the
    /// `len()` it last saw and passes it back here to get only what is new.
    pub fn since(&self, cursor: usize) -> &[Stamped] {
        &self.events[cursor.min(self.events.len())..]
    }

    /// The event that created chain `id`, if it was recorded.
    pub fn creation_of(&self, id: ChainId) -> Option<&Stamped> {
        self.events.iter().find(|s| s.event.id() == id)
    }

    /// The chains that came from `root`, as a tree in reading order (each parent
    /// before its children), at most `max_rows` rows, `root` first. A chain with two
    /// parents appears under the first one that reaches it.
    pub fn lineage(&self, root: ChainId, max_rows: usize) -> Vec<LineageRow> {
        let mut children: std::collections::HashMap<ChainId, Vec<&Stamped>> = std::collections::HashMap::new();
        for s in &self.events {
            for p in s.event.parents() {
                children.entry(p).or_default().push(s);
            }
        }
        let mut rows = Vec::new();
        let mut seen = vec![root];
        let mut stack: Vec<(ChainId, usize)> = vec![(root, 0)];
        while let Some((id, depth)) = stack.pop() {
            if rows.len() >= max_rows {
                break;
            }
            let Some(created) = self.creation_of(id) else { continue };
            rows.push(LineageRow { id, depth, step: created.step, how: created.event.how(), sequence: created.event.sequence().to_string() });
            let kids = children.get(&id).map(Vec::as_slice).unwrap_or(&[]);
            for kid in kids.iter().rev() {
                let kid_id = kid.event.id();
                if !seen.contains(&kid_id) {
                    seen.push(kid_id);
                    stack.push((kid_id, depth + 1));
                }
            }
        }
        rows
    }

    /// The protein chains that were read off a strand, with the step they appeared,
    /// oldest first: the roots of the protein lineages.
    pub fn translated_roots(&self) -> Vec<(ChainId, u64)> {
        self.events.iter().filter(|s| matches!(s.event, ChainEvent::Translated { .. })).map(|s| (s.event.id(), s.step)).collect()
    }

    /// The chain's ancestors, nearest first, each listed once.
    pub fn ancestors(&self, id: ChainId) -> Vec<ChainId> {
        let mut seen = Vec::new();
        let mut queue = vec![id];
        while !queue.is_empty() {
            let current = queue.remove(0);
            let Some(created) = self.creation_of(current) else { continue };
            for parent in created.event.parents() {
                if !seen.contains(&parent) {
                    seen.push(parent);
                    queue.push(parent);
                }
            }
        }
        seen
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// a (1) is generated, b (2) is a mutation of a, c (3) is generated, d (4) is b x c.
    fn sample() -> EventLog {
        let mut log = EventLog::new();
        let (a, b, c, d) = (log.new_id(), log.new_id(), log.new_id(), log.new_id());
        log.record(1, ChainEvent::Generated { id: a, sequence: "MET".into() });
        log.record(
            2,
            ChainEvent::Mutated {
                id: b,
                parent: a,
                mutation: Mutation::Substitution { index: 1, from: 'E', to: 'A' },
                sequence: "MAT".into(),
            },
        );
        log.record(3, ChainEvent::Generated { id: c, sequence: "KS".into() });
        log.record(4, ChainEvent::Recombined { id: d, parents: (b, c), cut: (2, 0), sequence: "MAKS".into() });
        log
    }

    #[test]
    fn ids_are_unique_and_increasing() {
        let mut log = EventLog::new();
        let ids: Vec<_> = (0..5).map(|_| log.new_id()).collect();
        assert_eq!(ids, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn events_report_their_child_sequence_and_parents() {
        let log = sample();
        let all = log.since(0);
        assert_eq!(all[0].event.parents(), Vec::<ChainId>::new());
        assert_eq!(all[1].event.parents(), vec![1]);
        assert_eq!(all[3].event.parents(), vec![2, 3]);
        assert_eq!(all[3].event.id(), 4);
        assert_eq!(all[3].event.sequence(), "MAKS");
    }

    #[test]
    fn since_returns_only_new_events() {
        let log = sample();
        assert_eq!(log.since(0).len(), 4);
        assert_eq!(log.since(3).len(), 1);
        assert_eq!(log.since(4).len(), 0);
        assert_eq!(log.since(99).len(), 0, "a cursor past the end is not an error");
    }

    #[test]
    fn creation_lookup_finds_the_right_event() {
        let log = sample();
        assert_eq!(log.creation_of(2).map(|s| s.step), Some(2));
        assert!(log.creation_of(UNRECORDED).is_none(), "id 0 never has a creation event");
        assert!(log.creation_of(42).is_none());
    }

    #[test]
    fn ancestors_walk_through_both_parents() {
        let log = sample();
        let mut found = log.ancestors(4);
        found.sort();
        assert_eq!(found, vec![1, 2, 3]);
        assert!(log.ancestors(1).is_empty());
    }

    #[test]
    fn ancestors_list_a_shared_ancestor_once() {
        let mut log = EventLog::new();
        let root = log.new_id();
        log.record(0, ChainEvent::Generated { id: root, sequence: "A".into() });
        let (x, y, z) = (log.new_id(), log.new_id(), log.new_id());
        let m = Mutation::Insertion { index: 0, residue: 'C' };
        log.record(1, ChainEvent::Mutated { id: x, parent: root, mutation: m, sequence: "CA".into() });
        log.record(1, ChainEvent::Mutated { id: y, parent: root, mutation: m, sequence: "CA".into() });
        log.record(2, ChainEvent::Recombined { id: z, parents: (x, y), cut: (1, 0), sequence: "CCA".into() });
        let ancestors = log.ancestors(z);
        assert_eq!(ancestors.iter().filter(|&&a| a == root).count(), 1);
        assert_eq!(ancestors.len(), 3);
    }

    #[test]
    fn a_lineage_lists_the_descendants_of_a_translated_protein() {
        let mut log = EventLog::new();
        let root = log.new_id();
        log.record(5, ChainEvent::Translated { id: root, gene: "AUGGCUUAA".into(), sequence: "MA".into() });
        let (child, grand, other) = (log.new_id(), log.new_id(), log.new_id());
        let m = Mutation::Insertion { index: 0, residue: 'C' };
        log.record(6, ChainEvent::Mutated { id: child, parent: root, mutation: m, sequence: "CMA".into() });
        log.record(7, ChainEvent::Mutated { id: grand, parent: child, mutation: m, sequence: "CCMA".into() });
        log.record(8, ChainEvent::Generated { id: other, sequence: "K".into() });
        let rows = log.lineage(root, 10);
        let shape: Vec<(ChainId, usize, &str)> = rows.iter().map(|r| (r.id, r.depth, r.how)).collect();
        assert_eq!(shape, vec![(root, 0, "translated"), (child, 1, "mutated"), (grand, 2, "mutated")]);
        assert_eq!(log.lineage(root, 2).len(), 2, "capped");
        assert!(log.lineage(999, 5).is_empty());
        assert_eq!(log.translated_roots(), vec![(root, 5)]);
    }
}
