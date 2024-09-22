use num::Float;

pub fn max_if<It, F, P, B>(it: It, mut func: F, mut pred: P) -> Option<It::Item>
where
    It: Iterator,
    F: FnMut(&It::Item) -> B,
    P: FnMut(&It::Item, &B) -> bool,
    B: PartialOrd,
{
    it.filter_map(|item| {
        let value = func(&item);
        if pred(&item, &value) {
            Some((item, value))
        } else {
            None
        }
    })
    .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
    .map(|item| item.0)
}

// trait AvgNum: Num + Copy + NumCast {}

#[derive(Debug)]
pub struct Avg<T: Float> {
    sum: T,
    count: usize,
}

impl<T: Float> Avg<T> {
    pub fn new() -> Self {
        Avg {
            sum: T::zero(),
            count: 0,
        }
    }

    pub fn add(&mut self, x: T) {
        self.sum = self.sum + x;
        self.count += 1;
    }

    pub fn get(&self) -> T {
        if self.count == 0 {
            T::zero()
        } else {
            self.sum / T::from(self.count).unwrap()
        }
    }
}

impl<T: Float> Default for Avg<T> {
    fn default() -> Self {
        Avg::new()
    }
}

#[cfg(test)]
mod avg_test {
    use super::Avg;

    #[test]
    fn avg() {
        let mut avg: Avg<f64> = Avg::new();
        avg.add(2.0);
        avg.add(5.0);
        avg.add(7.0);
        avg.add(2.0);
        assert_eq!(avg.get(), 4.0);
    }
}
