pub fn max_if<It, F, P, B>(it: It, mut func: F, mut pred: P) -> Option<It::Item>
where
    It: Iterator,
    F: FnMut(&It::Item) -> B,
    P: FnMut(&B) -> bool,
    B: PartialOrd,
{
    it.filter_map(|item| {
        let value = func(&item);
        if pred(&value) {
            Some((item, value))
        } else {
            None
        }
    })
    .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
    .map(|item| item.0)
}
