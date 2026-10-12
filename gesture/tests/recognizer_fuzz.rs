use std::time::Duration;

use geometry::{Point, Vector};
use gesture::{
    ArenaDecision, ArenaDecisions, ClickConfig, ClickRecognizer, DragAxis, DragConfig,
    DragRecognizer, GestureArena, GestureOutput, LongPressRecognizer, PinchRecognizer, Recognizer,
};
use input::{GesturePhase, PointerButton, PointerEvent, PointerId, TouchpadGesture};
use proptest::prelude::*;

const PINCH_MEMBER: usize = 5;

#[derive(Debug, Clone, Copy)]
enum Step {
    Down {
        x: f32,
        y: f32,
        button: PointerButton,
    },
    Move {
        dx: f32,
        dy: f32,
    },
    Up,
    Cancel,
    Poll,
    Pinch(GesturePhase, f32),
}

fn button() -> impl Strategy<Value = PointerButton> {
    prop_oneof![
        4 => Just(PointerButton::Primary),
        1 => Just(PointerButton::Secondary),
    ]
}

fn phase() -> impl Strategy<Value = GesturePhase> {
    prop_oneof![
        Just(GesturePhase::Begin),
        Just(GesturePhase::Update),
        Just(GesturePhase::End),
        Just(GesturePhase::Cancel),
    ]
}

fn timed_step() -> impl Strategy<Value = (u64, Step)> {
    let step = prop_oneof![
        2 => (0.0..200.0f32, 0.0..200.0f32, button()).prop_map(|(x, y, button)| Step::Down { x, y, button }),
        5 => (-12.0..12.0f32, -12.0..12.0f32).prop_map(|(dx, dy)| Step::Move { dx, dy }),
        2 => Just(Step::Up),
        1 => Just(Step::Cancel),
        2 => Just(Step::Poll),
        1 => (phase(), 0.5..2.0f32).prop_map(|(phase, scale)| Step::Pinch(phase, scale)),
    ];
    (0..300u64, step)
}

struct Harness {
    dispatch_before_resolving: bool,
    recognizers: Vec<Box<dyn Recognizer>>,
    arena: GestureArena<usize>,
    winner: Option<usize>,
    members: Vec<usize>,
    decided: Vec<usize>,
    pressed_button: Option<PointerButton>,
    position: Point,
    now: Duration,
    outputs: Vec<GestureOutput>,
}

impl Harness {
    fn new(dispatch_before_resolving: bool) -> Self {
        let recognizers: Vec<Box<dyn Recognizer>> = vec![
            Box::new(ClickRecognizer::default()),
            Box::new(ClickRecognizer::new(ClickConfig {
                button: PointerButton::Secondary,
                ..ClickConfig::default()
            })),
            Box::new(LongPressRecognizer::default()),
            Box::new(DragRecognizer::default()),
            Box::new(DragRecognizer::new(DragConfig {
                axis: DragAxis::Vertical,
                ..DragConfig::default()
            })),
            Box::new(PinchRecognizer::new()),
        ];
        Self {
            dispatch_before_resolving,
            recognizers,
            arena: GestureArena::new(),
            winner: None,
            members: Vec::new(),
            decided: Vec::new(),
            pressed_button: None,
            position: Point::ZERO,
            now: Duration::ZERO,
            outputs: Vec::new(),
        }
    }

    fn event(&self, button: Option<PointerButton>) -> PointerEvent {
        PointerEvent {
            button,
            timestamp: self.now,
            ..PointerEvent::mouse(self.position)
        }
    }

    fn apply(&mut self, decisions: ArenaDecisions<usize>) -> Result<(), TestCaseError> {
        for decision in decisions {
            match decision {
                ArenaDecision::Won(member) => {
                    prop_assert!(self.winner.is_none(), "second winner {member}");
                    self.winner = Some(member);
                    self.decided.push(member);
                    self.recognizers[member].accepted();
                }
                ArenaDecision::Lost(member) => {
                    self.decided.push(member);
                    self.recognizers[member].rejected();
                }
            }
        }
        Ok(())
    }

    fn vote_all(
        &mut self,
        vote: impl Fn(&mut dyn Recognizer) -> gesture::Vote,
    ) -> Result<(), TestCaseError> {
        if !self.dispatch_before_resolving {
            for member in 0..self.recognizers.len() {
                let vote = vote(self.recognizers[member].as_mut());
                let decisions = self.arena.resolve(PointerId::MOUSE, member, vote);
                self.apply(decisions)?;
            }
            return Ok(());
        }
        let votes: Vec<_> = self
            .recognizers
            .iter_mut()
            .map(|recognizer| vote(recognizer.as_mut()))
            .collect();
        for (member, vote) in votes.into_iter().enumerate() {
            let decisions = self.arena.resolve(PointerId::MOUSE, member, vote);
            self.apply(decisions)?;
        }
        Ok(())
    }

    fn check_outputs(&mut self) -> Result<(), TestCaseError> {
        for member in 0..self.recognizers.len() {
            self.outputs.clear();
            self.recognizers[member].drain_outputs(&mut self.outputs);
            if member != PINCH_MEMBER && !self.outputs.is_empty() {
                prop_assert_eq!(
                    self.winner,
                    Some(member),
                    "outputs {:?} from a non-winner",
                    self.outputs
                );
            }
        }
        Ok(())
    }

    fn finish_press(&mut self, decisions: ArenaDecisions<usize>) -> Result<(), TestCaseError> {
        self.apply(decisions)?;
        let released = self.arena.release(PointerId::MOUSE);
        self.apply(released)?;
        self.check_outputs()?;
        let mut decided = self.decided.clone();
        decided.sort_unstable();
        prop_assert_eq!(
            &decided,
            &self.members,
            "each member needs exactly one decision"
        );
        self.pressed_button = None;
        Ok(())
    }

    fn step(&mut self, elapsed: u64, step: Step) -> Result<(), TestCaseError> {
        self.now += Duration::from_millis(elapsed);
        match step {
            Step::Down { x, y, button } => {
                if self.pressed_button.is_some() {
                    return Ok(());
                }
                self.position = Point::new(x, y);
                let event = self.event(Some(button));
                self.winner = None;
                self.decided.clear();
                self.members = (0..self.recognizers.len())
                    .filter(|&member| self.recognizers[member].allows_pointer(&event))
                    .collect();
                for &member in &self.members {
                    prop_assert!(self.arena.add(PointerId::MOUSE, member));
                }
                self.pressed_button = Some(button);
                self.vote_all(|recognizer| recognizer.pointer_down(&event))?;
                let closed = self.arena.close(PointerId::MOUSE);
                self.apply(closed)?;
            }
            Step::Move { dx, dy } => {
                self.position = self.position + Vector::new(dx, dy);
                let event = self.event(None);
                self.vote_all(|recognizer| recognizer.pointer_move(&event))?;
            }
            Step::Up => {
                if let Some(button) = self.pressed_button {
                    let event = self.event(Some(button));
                    self.vote_all(|recognizer| recognizer.pointer_up(&event))?;
                    let swept = self.arena.sweep(PointerId::MOUSE);
                    self.finish_press(swept)?;
                }
            }
            Step::Cancel => {
                if self.pressed_button.is_some() {
                    for recognizer in &mut self.recognizers {
                        recognizer.pointer_cancel();
                    }
                    self.finish_press(ArenaDecisions::new())?;
                }
            }
            Step::Poll => {
                let now = self.now;
                self.vote_all(|recognizer| recognizer.poll(now))?;
            }
            Step::Pinch(phase, scale) => {
                let gesture = TouchpadGesture::Pinch {
                    phase,
                    scale,
                    rotation: 0.0,
                    delta: Vector::ZERO,
                };
                for recognizer in &mut self.recognizers {
                    if recognizer.touchpad(&gesture) == gesture::Vote::Accept {
                        recognizer.accepted();
                    }
                }
            }
        }
        self.check_outputs()
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn at_most_one_recognizer_wins_each_press(
        dispatch_before_resolving in any::<bool>(),
        steps in prop::collection::vec(timed_step(), 1..80),
    ) {
        let mut harness = Harness::new(dispatch_before_resolving);
        for (elapsed, step) in steps {
            harness.step(elapsed, step)?;
        }
        if harness.pressed_button.is_some() {
            harness.step(0, Step::Up)?;
        }
    }
}
