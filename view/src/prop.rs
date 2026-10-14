use geometry::Color;
use reactive::{Memo, Runtime, Signal};

/// A property value: fixed at build time, or recomputed whenever the signals it reads change.
pub enum Prop<T> {
    /// A literal.
    Static(T),
    /// Recomputed reactively.
    Dynamic(Box<dyn FnMut(&mut Runtime) -> T>),
}

/// A closure that derives a property from signals; build one with [`bind`].
pub struct Bind<F>(F);

/// Derives a property from signals: `.bg(bind(move |rt| if on.get(rt) { GREEN } else { GRAY }))`.
pub fn bind<T, F: FnMut(&mut Runtime) -> T + 'static>(compute: F) -> Bind<F> {
    Bind(compute)
}

/// Converts literals, signals, memos and [`bind`] closures into a [`Prop`].
pub trait IntoProp<T> {
    /// Performs the conversion.
    fn into_prop(self) -> Prop<T>;
}

impl<T> IntoProp<T> for Prop<T> {
    fn into_prop(self) -> Prop<T> {
        self
    }
}

impl<T: Clone + 'static> IntoProp<T> for Signal<T> {
    fn into_prop(self) -> Prop<T> {
        Prop::Dynamic(Box::new(move |runtime| self.get(runtime)))
    }
}

impl<T: Clone + 'static> IntoProp<T> for Memo<T> {
    fn into_prop(self) -> Prop<T> {
        Prop::Dynamic(Box::new(move |runtime| self.get(runtime)))
    }
}

impl<T, F: FnMut(&mut Runtime) -> T + 'static> IntoProp<T> for Bind<F> {
    fn into_prop(self) -> Prop<T> {
        Prop::Dynamic(Box::new(self.0))
    }
}

macro_rules! literal_props {
    ($target:ty: $($source:ty => $convert:expr),+ $(,)?) => {
        $(impl IntoProp<$target> for $source {
            fn into_prop(self) -> Prop<$target> {
                let convert: fn($source) -> $target = $convert;
                Prop::Static(convert(self))
            }
        })+
    };
}

literal_props!(f32: f32 => |value| value, i32 => |value| value as f32, f64 => |value| value as f32, u32 => |value| value as f32);
literal_props!(String: String => |value| value, &str => str::to_owned, &String => Clone::clone);
literal_props!(Color: Color => |value| value);
literal_props!(bool: bool => |value| value);

impl<T> Prop<T> {
    /// Maps the produced value.
    pub fn map<U: 'static>(self, mut transform: impl FnMut(T) -> U + 'static) -> Prop<U>
    where
        T: 'static,
    {
        match self {
            Self::Static(value) => Prop::Static(transform(value)),
            Self::Dynamic(mut compute) => {
                Prop::Dynamic(Box::new(move |runtime| transform(compute(runtime))))
            }
        }
    }
}
