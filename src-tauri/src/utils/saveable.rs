use ski_analyzer_lib::error::Result;

pub trait Saver<T> {
    fn save(&mut self, data: &T) -> Result<()>;
    fn load(&self) -> Result<T>;
}

pub struct Saveable<T, S> {
    saver: S,
    data: T,
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

    pub fn get(&self) -> &T {
        &self.data
    }

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
    T: Default,
{
    owner: &'a mut Saveable<T, S>,
}

impl<'a, T, S> ModifyingSaveable<'a, T, S>
where
    S: Saver<T>,
    T: Default,
{
    pub fn get(&mut self) -> &mut T {
        &mut self.owner.data
    }

    pub fn set(&mut self, value: T) {
        self.owner.data = value;
    }
}

impl<'a, T, S> Drop for ModifyingSaveable<'a, T, S>
where
    S: Saver<T>,
    T: Default,
{
    fn drop(&mut self) {
        self.owner.save();
    }
}
