use std::sync::{Arc, Mutex};
use std::marker::Send;

use specs::Entity;

#[derive(Debug)]
pub enum Event {
    Log(Entity, String),
} 

pub trait Listener {
    fn notify(&mut self, event: &Event);
}

pub struct Dispatcher<'a>
{
    listeners: Vec<Arc<Mutex<dyn Listener + Send + 'a>>>,
}

unsafe impl<'a> Send for Dispatcher<'a> {}

impl<'a> Dispatcher<'a>
{
    pub fn new() -> Dispatcher<'a> {
        Dispatcher { listeners: Vec::new() }
    }

    pub fn dispatch(&mut self, event: Event) {
        for l in self.listeners.iter() {
                if let Ok(ref mut listener) = l.lock() {
                    listener.notify(&event);
                }
        }
    }

    pub fn register_listener(&mut self, listener: Arc<Mutex<dyn Listener + Send + 'a>>) {
        self.listeners.push(listener);
    }
}
