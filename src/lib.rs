use std::any::TypeId;
use std::collections::HashMap;
use std::mem;

pub struct PortableTypedEventHandler(TypeId, Box<FnMut(&mut ())>);

pub struct TypedEventRouter {
    topics: HashMap<TypeId, Vec<Box<FnMut(&mut ())>>>
}

impl TypedEventRouter {
    pub fn new() -> TypedEventRouter {
        TypedEventRouter {
            topics: HashMap::new(),
        }
    }
    
    pub fn subscribe<T: 'static>(&mut self, f: impl FnMut(&mut T)) {
        let f = Self::make_portable_handler(f);
        self.subscribe_portable(f);
    }
    
    pub fn emit<T: 'static>(&mut self, event: &mut T) {
        for listener in self.topics.get_mut(&TypeId::of::<T>()).unwrap() {
            let f: &mut Box<FnMut(&mut T)> = unsafe {
                mem::transmute(listener)
            };
            f(event);
        }
    }

    pub fn make_portable_handler<T: 'static>(f: impl FnMut(&mut T)) -> PortableTypedEventHandler {
        let f: Box<FnMut(&mut T)> = Box::new(f); // Need this or compiler cries
        PortableTypedEventHandler(TypeId::of::<T>(), unsafe { mem::transmute(f) })
    }

    pub fn subscribe_portable(&mut self, f: PortableTypedEventHandler) {
        let PortableTypedEventHandler(type_id, f) = f;
        let listeners = self.topics.entry(type_id).or_default();
        listeners.push(f);
    }
}

#[cfg(test)]
mod tests {
    use super::TypedEventRouter;

    struct Foo { x: i32 }
    struct Bar { y: i32 }
    struct Baz { z: i32 }

    #[test]
    fn it_works() {
	let mut r = TypedEventRouter::new();
	r.subscribe(|e: &mut Foo| { e.x += 1; });
	r.subscribe(|e: &mut Foo| { e.x += 2; });
	
	r.subscribe(|e: &mut Bar| { e.y += 1; });
	r.subscribe(|e: &mut Bar| { e.y += 2; });
	
	r.subscribe(|e: &mut Baz| { e.z += 1; });
	r.subscribe(|e: &mut Baz| { e.z += 2; });

        let ref mut foo = Foo { x: 42 };
        let ref mut bar = Bar { y: 24 };
        let ref mut baz = Baz { z: 92 };
	
	r.emit(foo);
	r.emit(baz);
	r.emit(bar);

        assert!(foo.x == 45);
        assert!(bar.y == 27);
        assert!(baz.z == 95);
    }
}

