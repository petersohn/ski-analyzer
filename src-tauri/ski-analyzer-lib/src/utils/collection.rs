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
pub struct Avg {
    sum: f64,
    amount: f64,
}

impl Avg {
    pub fn new() -> Self {
        Avg {
            sum: 0.0,
            amount: 0.0,
        }
    }

    pub fn add(&mut self, x: f64) {
        self.add2(x, 1.0);
    }

    pub fn add2(&mut self, x: f64, y: f64) {
        assert!(y > 0.0);
        self.sum += x;
        self.amount += y;
    }

    pub fn remove(&mut self, x: f64) {
        self.remove2(x, 1.0);
    }

    pub fn remove2(&mut self, x: f64, y: f64) {
        assert!(y > 0.0);
        assert!(self.amount >= y);
        self.sum = self.sum - x;
        self.amount -= y;
    }

    pub fn get(&self) -> f64 {
        if self.amount == 0.0 {
            0.0
        } else {
            self.sum / self.amount as f64
        }
    }
}

impl Default for Avg {
    fn default() -> Self {
        Avg::new()
    }
}

#[cfg(test)]
mod avg_test {
    use super::Avg;

    #[test]
    fn empty() {
        let avg = Avg::new();
        assert_eq!(avg.get(), 0.0);
    }

    #[test]
    fn add() {
        let mut avg = Avg::new();
        avg.add(2.0);
        avg.add(5.0);
        avg.add(7.0);
        avg.add(2.0);
        assert_eq!(avg.get(), 4.0);
    }

    #[test]
    fn remove() {
        let mut avg = Avg::new();
        avg.add(1.0);
        avg.add(5.0);
        avg.add(4.0);
        avg.add(2.0);
        avg.add(1.0);
        avg.remove(2.0);
        avg.remove(5.0);
        avg.add(6.0);
        assert_eq!(avg.get(), 3.0);
    }

    #[test]
    fn add2() {
        let mut avg = Avg::new();
        avg.add2(5.0, 1.0);
        avg.add2(5.0, 0.5);
        avg.add2(10.0, 1.0);
        avg.add2(20.0, 2.0);
        avg.add2(15.0, 1.0);
        assert_eq!(avg.get(), 10.0);
    }

    #[test]
    fn remove2() {
        let mut avg = Avg::new();
        avg.add2(20.0, 5.0);
        avg.add2(10.0, 2.0);
        avg.remove2(20.0, 5.0);
        avg.add2(5.0, 1.0);
        assert_eq!(avg.get(), 5.0);
    }
}
