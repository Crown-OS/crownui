use anyhow::{Result, bail};
use geometry::Color;
use reactive::Signal;
use view::{AnyView, Cx, Styled, View, bind, for_each};
use widgets::{button, hstack, text, vstack};

/// Nodes one row costs: the stack, two texts, the button and its label.
const ROW_NODES: usize = 5;
const DEEP_LEVELS: usize = 48;
const MIXED_DEPTH: usize = 6;
const HOVER_BACKGROUND: Color = Color::new(0.2, 0.5, 1.0, 0.4);

/// The shape of a benchmark tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Family {
    /// One column of rows: shallow and broad.
    Wide,
    /// Rows nested 48 levels deep.
    Deep,
    /// Seeded random branching and depth.
    Mixed,
}

impl Family {
    pub(crate) const ALL: [Family; 3] = [Family::Wide, Family::Deep, Family::Mixed];

    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Wide => "wide",
            Self::Deep => "deep",
            Self::Mixed => "mixed",
        }
    }

    pub(crate) fn parse(name: &str) -> Result<Self> {
        match name.trim() {
            "wide" => Ok(Self::Wide),
            "deep" => Ok(Self::Deep),
            "mixed" => Ok(Self::Mixed),
            other => bail!("unknown family {other:?}; expected wide, deep or mixed"),
        }
    }
}

/// The signals the scenarios drive.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Handles {
    /// The first row's text.
    pub(crate) text: Signal<String>,
    /// Rows appended after the family tree.
    pub(crate) rows: Signal<Vec<u32>>,
}

/// Builds a `family` tree of about `size` nodes: a first row whose text is a signal, the family
/// body, then rows driven by a list signal.
pub(crate) fn build(cx: &mut Cx, family: Family, size: usize, seed: u64) -> (AnyView, Handles) {
    let handles = Handles {
        text: cx.signal("Row 0".to_owned()),
        rows: cx.signal(Vec::new()),
    };
    let mut builder = Builder {
        rng: Rng(seed | 1),
        next_row: 1,
        budget: size.saturating_sub(ROW_NODES + 3),
    };
    let body = match family {
        Family::Wide => builder.wide(),
        Family::Deep => builder.deep(),
        Family::Mixed => builder.mixed(),
    };
    let first = hstack((text(handles.text), hover_button())).boxed();
    let appended = for_each(
        handles.rows,
        |row| *row,
        |_, item| {
            hstack((
                text(bind(move |runtime| format!("Added {}", item.get(runtime)))),
                hover_button(),
            ))
        },
    );
    (vstack((first, body, appended)).boxed(), handles)
}

fn hover_button() -> impl View {
    button("Open").hover(|style| style.bg(HOVER_BACKGROUND))
}

struct Builder {
    rng: Rng,
    next_row: usize,
    budget: usize,
}

impl Builder {
    fn row(&mut self) -> AnyView {
        let index = self.next_row;
        self.next_row += 1;
        self.budget = self.budget.saturating_sub(ROW_NODES);
        hstack((
            text(format!("Row {index}")),
            text(format!("{} items", index * 7 % 100)),
            hover_button(),
        ))
        .boxed()
    }

    fn rows(&mut self, count: usize) -> Vec<AnyView> {
        (0..count).map(|_| self.row()).collect()
    }

    fn wide(&mut self) -> AnyView {
        let count = self.budget / ROW_NODES;
        vstack(self.rows(count)).boxed()
    }

    fn deep(&mut self) -> AnyView {
        let per_level = (self.budget / ROW_NODES / DEEP_LEVELS).max(1);
        self.deep_level(DEEP_LEVELS, per_level)
    }

    fn deep_level(&mut self, levels: usize, per_level: usize) -> AnyView {
        let rows = self.rows(per_level);
        let nested = (levels > 1).then(|| self.deep_level(levels - 1, per_level));
        vstack((rows, nested)).boxed()
    }

    fn mixed(&mut self) -> AnyView {
        let mut sections = Vec::new();
        while self.budget >= ROW_NODES {
            sections.push(self.mixed_section(MIXED_DEPTH));
        }
        vstack(sections).boxed()
    }

    fn mixed_section(&mut self, depth: usize) -> AnyView {
        if self.budget < ROW_NODES {
            self.budget = 0;
            return text("·").boxed();
        }
        if depth == 0 || self.budget < ROW_NODES * 2 {
            return self.row();
        }
        let width = 1 + self.rng.below(5) as usize;
        let children: Vec<AnyView> = (0..width)
            .map(|_| {
                if self.rng.below(3) == 0 {
                    self.row()
                } else {
                    self.mixed_section(depth - 1)
                }
            })
            .collect();
        self.budget = self.budget.saturating_sub(1);
        if self.rng.below(2) == 0 {
            vstack(children).boxed()
        } else {
            hstack(children).boxed()
        }
    }
}

/// xorshift64*: deterministic and dependency free.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        let mut state = self.0;
        state ^= state >> 12;
        state ^= state << 25;
        state ^= state >> 27;
        self.0 = state;
        state.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn below(&mut self, bound: u64) -> u64 {
        self.next() % bound
    }
}
