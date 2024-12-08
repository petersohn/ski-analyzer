use super::saveable::{Saveable, Saver};
use serde::{de::DeserializeOwned, Serialize};
use ski_analyzer_lib::error::{Error, ErrorType, Result};
use std::cell::RefCell;
use std::rc::Rc;

struct DummySaver<T> {
    value: Rc<RefCell<Option<T>>>,
}

impl<T> Saver<T> for DummySaver<T>
where
    T: Serialize + DeserializeOwned + Clone,
{
    fn save(&mut self, data: &T) -> Result<()> {
        self.value.replace(Some(data.clone()));
        Ok(())
    }

    fn load(&self) -> Result<T> {
        match &*self.value.borrow() {
            None => Err(Error::new_s(ErrorType::InputError, "whatever")),
            Some(x) => Ok(x.clone()),
        }
    }
}

#[test]
fn load_and_replace() {
    let value = Rc::new(RefCell::new(Some(2)));
    let mut saveable = Saveable::new(DummySaver {
        value: Rc::clone(&value),
    });
    assert_eq!(*saveable.get(), 2);
    let old = saveable.modify().replace(10);
    assert_eq!(old, 2);
    assert_eq!(*value.borrow(), Some(10));
    assert_eq!(*saveable.get(), 10);
}

#[test]
fn default_and_replace() {
    let value: Rc<RefCell<Option<u32>>> = Rc::new(RefCell::new(None));
    let mut saveable = Saveable::new(DummySaver {
        value: Rc::clone(&value),
    });
    assert_eq!(*saveable.get(), 0);
    let old = saveable.modify().replace(3);
    assert_eq!(old, 0);
    assert_eq!(*value.borrow(), Some(3));
    assert_eq!(*saveable.get(), 3);
}

#[test]
fn modify() {
    let value = Rc::new(RefCell::new(Some(vec![10, 12])));
    let saver: DummySaver<Vec<u32>> = DummySaver {
        value: Rc::clone(&value),
    };
    let mut saveable = Saveable::new(saver);
    assert_eq!(*saveable.get(), vec![10, 12]);
    saveable.modify().push(412);
    assert_eq!(*value.borrow(), Some(vec![10, 12, 412]));
    assert_eq!(*saveable.get(), vec![10, 12, 412]);
}
