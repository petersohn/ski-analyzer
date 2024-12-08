use ski_analyzer_lib::error::Result;

pub trait Saver<T> {
    fn save(&mut self, data: &T) -> Result<()>;
    fn load(&self) -> Result<T>;
}

pub struct Saveable<T, S> {
    saver: S,
    data: T,
}

impl<T, S> Saveable<T, S> {
    pub fn get(&self) -> &T {
        &self.data
    }
}

impl<T, S> Saveable<T, S>
where
    S: Saver<T>,
    T: Default,
{
    pub fn new(saver: S) -> Self {
        let res = saver.load();
        let data = match res {
            Ok(d) => d,
            Err(err) => {
                eprintln!("Failed to load: {}", err);
                T::default()
            }
        };
        Self { saver, data }
    }
}

impl<T, S> Saveable<T, S>
where
    S: Saver<T>,
{
    pub fn modify(&mut self) -> ModifyingSaveable<'_, T, S> {
        ModifyingSaveable { owner: self }
    }

    fn save(&mut self) {
        if let Err(err) = self.saver.save(&self.data) {
            eprintln!("Failed to save: {}", err);
        }
    }
}

pub struct ModifyingSaveable<'a, T, S>
where
    S: Saver<T>,
{
    owner: &'a mut Saveable<T, S>,
}

impl<'a, T, S> ModifyingSaveable<'a, T, S>
where
    S: Saver<T>,
{
    pub fn replace(&mut self, value: T) -> T {
        std::mem::replace(&mut self.owner.data, value)
    }
}

impl<'a, T, S> Drop for ModifyingSaveable<'a, T, S>
where
    S: Saver<T>,
{
    fn drop(&mut self) {
        self.owner.save();
    }
}

impl<'a, T, S> std::ops::Deref for ModifyingSaveable<'a, T, S>
where
    S: Saver<T>,
{
    type Target = T;

    fn deref(&self) -> &T {
        &self.owner.data
    }
}

impl<'a, T, S> std::ops::DerefMut for ModifyingSaveable<'a, T, S>
where
    S: Saver<T>,
{
    fn deref_mut(&mut self) -> &mut T {
        &mut self.owner.data
    }
}
