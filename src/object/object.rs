pub trait Object<'a, T>{
    fn get(&'a self, name: &str) -> Option<&'a T>;
    fn get_mut(&'a mut self, name: &str) -> Option<&'a mut T>;
    fn add(&mut self, name: &str, value: T);
}