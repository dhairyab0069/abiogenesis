//! Turning logged events into row changes.

use super::ChainView;
use super::row::Row;
use super::timing::MAX_ROWS;
use crate::events::{ChainEvent, ChainId, UNRECORDED};

impl ChainView {
    fn row_index(&self, id: ChainId) -> Option<usize> {
        if id == UNRECORDED {
            return None;
        }
        self.rows.iter().position(|r| r.id == id)
    }

    fn push(&mut self, row: Row) {
        self.rows.push(row);
        if self.rows.len() > MAX_ROWS {
            self.rows.remove(0);
        }
    }

    pub(super) fn apply_event(&mut self, event: &ChainEvent) {
        match event {
            ChainEvent::Generated { id, sequence } => self.push(Row::growing(*id, sequence)),
            ChainEvent::Mutated { id, parent, mutation, sequence } => match self.row_index(*parent) {
                // The child takes the parent's place, as it does in the soup.
                Some(i) => {
                    let row = &mut self.rows[i];
                    row.id = *id;
                    if !row.apply(*mutation) || row.sequence() != *sequence {
                        *row = Row::growing(*id, sequence);
                    }
                }
                None => self.push(Row::growing(*id, sequence)),
            },
            ChainEvent::Recombined { id, cut, sequence, .. } => {
                self.push(Row::recombined(*id, sequence, cut.0));
            }
        }
    }
}
