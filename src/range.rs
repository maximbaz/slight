use std::cmp::Ordering;

pub struct Range {
    curr: usize,
    max: usize,
}

impl Range {
    fn from_a_to_b(a: usize, b: usize) -> Box<dyn Iterator<Item = usize>> {
        let r: Box<dyn Iterator<Item = usize>> = match b.cmp(&a) {
            Ordering::Greater => Box::new(a..=b),
            Ordering::Less => Box::new((b..=a).rev()),
            Ordering::Equal => Box::new(std::iter::empty()),
        };
        r
    }

    fn by_percent(
        curr: usize,
        max: usize,
        diff_percent: f32,
        exponent: f32,
    ) -> Box<dyn Iterator<Item = usize>> {
        let r: Box<dyn Iterator<Item = usize>> = if diff_percent.is_sign_positive() {
            Box::new(
                Range::normalized(0, max)
                    .filter(move |&v| v > curr)
                    .take((diff_percent * exponent) as usize),
            )
        } else {
            Box::new(
                Range::normalized(0, max)
                    .filter(move |&v| v < curr)
                    .rev()
                    .take((diff_percent.copysign(1.0) * exponent) as usize),
            )
        };
        r
    }

    fn normalized(min: usize, max: usize) -> Box<dyn DoubleEndedIterator<Item = usize>> {
        let mut v = vec![];
        let diff = (max as f32 - min as f32) / 100.0;
        let mut n = min as f32 + diff;

        while n as usize <= max {
            v.push(n as usize);
            n += diff;
        }
        Box::new(v.into_iter())
    }
}

pub enum Step {
    To(Range),
    By(Range),
}

pub enum Value {
    Absolute(isize, Step),
    Relative(f32, Step),
}

pub struct Exponential {
    value: Value,
    exponent: f32,
}

pub trait RangeBuilder {
    fn build(&self) -> Box<dyn Iterator<Item = usize>>;
}

impl Range {
    pub fn new(curr: usize, max: usize) -> Self {
        Self { curr, max }
    }

    pub fn to(self) -> Step {
        Step::To(self)
    }

    pub fn by(self) -> Step {
        Step::By(self)
    }
}

impl Step {
    pub fn absolute(self, v: isize) -> Value {
        Value::Absolute(v, self)
    }

    pub fn relative(self, v: f32) -> Value {
        Value::Relative(v, self)
    }
}

impl Value {
    pub fn exp(self, exponent: f32) -> Exponential {
        Exponential {
            value: self,
            exponent,
        }
    }
}

impl RangeBuilder for Value {
    fn build(&self) -> Box<dyn Iterator<Item = usize>> {
        match *self {
            Value::Absolute(new, Step::To(Range { curr, .. })) => {
                let new = new as usize;
                Range::from_a_to_b(curr, new)
            }
            Value::Absolute(v, Step::By(Range { curr, .. })) => {
                let new = (curr as isize).checked_add(v).unwrap_or(0) as usize;
                Range::from_a_to_b(curr, new)
            }
            Value::Relative(percent, Step::To(Range { curr, max })) => {
                let new = (max as f32 / 100.0 * percent) as usize;
                Range::from_a_to_b(curr, new)
            }
            Value::Relative(percent, Step::By(Range { curr, max })) => {
                Range::by_percent(curr, max, percent, 1.0)
            }
        }
    }
}

impl RangeBuilder for Exponential {
    fn build(&self) -> Box<dyn Iterator<Item = usize>> {
        match self.value {
            Value::Absolute(new, Step::To(Range { curr, .. })) => {
                let new = new as usize;
                Range::from_a_to_b(curr, new)
            }
            Value::Absolute(v, Step::By(Range { curr, .. })) => {
                let new = (curr as isize).checked_add(v).unwrap_or(0) as usize;
                Range::from_a_to_b(curr, new)
            }
            Value::Relative(percent, Step::To(Range { curr, max })) => {
                let new = (max as f32 / 100.0 * percent) as usize;
                Range::from_a_to_b(curr, new)
            }
            Value::Relative(percent, Step::By(Range { curr, max })) => {
                Range::by_percent(curr, max, percent, self.exponent)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn range_normalized_len_eq_100() {
        assert_eq!(Range::normalized(0, 255).count(), 100);
    }
}
