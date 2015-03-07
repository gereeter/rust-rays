use rand::Rng;

pub trait Distribution: Sized {
    type Output;

    fn sample<R: Rng>(&self, rng: &mut R) -> Self::Output;
    
    fn map<O, F: Fn(Self::Output) -> O>(self, func: F) -> Map<Self, F> {
        Map {
            inner: self,
            func: func
        }
    }

    fn or<Other: Distribution<Output=Self::Output>>(self, other: Other, other_prob: f32) -> Or<Self, Other> {
        Or {
            inner_1: self,
            inner_2: other,
            prob_2: other_prob
        }
    }

    fn pair<Other: Distribution>(self, other: Other) -> Pair<Self, Other> {
        Pair {
            first: self,
            second: other
        }
    }
}

pub struct Map<Inner, F> {
    inner: Inner,
    func: F
}

impl<Inner: Distribution, O, F: Fn(Inner::Output) -> O> Distribution for Map<Inner, F> {
    type Output = O;

    fn sample<R: Rng>(&self, rng: &mut R) -> O {
        (self.func)(self.inner.sample(rng))
    }
}

pub struct Const<T> {
    value: T
}

impl<T> Const<T> {
    pub fn new(value: T) -> Const<T> {
        Const {
            value: value
        }
    }
}

impl<T: Clone> Distribution for Const<T> {
    type Output = T;

    fn sample<R: Rng>(&self, _: &mut R) -> T {
        self.value.clone()
    }
}

pub struct Or<A, B> {
    inner_1: A,
    inner_2: B,
    prob_2: f32
}

impl<A: Distribution, B: Distribution<Output=A::Output>> Distribution for Or<A, B> {
    type Output = A::Output;

    fn sample<R: Rng>(&self, rng: &mut R) -> A::Output {
        if rng.next_f32() < self.prob_2 {
            self.inner_2.sample(rng)
        } else {
            self.inner_1.sample(rng)
        }
    }
}

pub struct Pair<A, B> {
    first: A,
    second: B
}

impl<A: Distribution, B: Distribution> Distribution for Pair<A, B> {
    type Output = (A::Output, B::Output);

    fn sample<R: Rng>(&self, rng: &mut R) -> (A::Output, B::Output) {
        (self.first.sample(rng), self.second.sample(rng))
    }
}
