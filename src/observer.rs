use std::sync::{Weak, Arc, Mutex};

#[derive(Debug)]
pub enum Event {
    Log(String)
} 

pub trait Listener {
    fn notify(&mut self, event: &Event);
}

pub trait Dispatchable
{
    fn register_listener(&mut self, listener: Arc<Mutex<dyn Listener>>);
}

pub struct Dispatcher
{
    /// A list of synchronous weak refs to listeners
    listeners: Vec<Weak<Mutex<dyn Listener>>>,
}

impl Dispatchable for Dispatcher
{
    /// Registers a new listener
    fn register_listener(&mut self, listener: Arc<Mutex<dyn Listener>>) {
        self.listeners.push(Arc::downgrade(&listener));
    }
}

impl Dispatcher
{
    pub fn new() -> Dispatcher {
        Dispatcher { listeners: Vec::new() }
    }

    pub fn num_listeners(&self) -> usize {
        self.listeners.len()
    }

    pub fn dispatch(&mut self, event: Event) {
        let mut cleanup = false;
        // Call the listeners
        for l in self.listeners.iter() {
            if let Some(mut listener_rc) = l.upgrade() {
                if let Ok(mut listener) = listener_rc.lock(){
                    listener.notify(&event);
                } else {
                    cleanup = true;
                }
            } else {
                cleanup = true;
            }
        }
        // If there were invalid weak refs, clean up the list
        if cleanup {
            self.listeners.retain(|ref l| {
                // Only retain valid weak refs
                let got_ref = l.clone().upgrade();
                match got_ref {
                    None => false,
                    _ => true,
                }
            });
        }
    }
}

// ------------------ TEST -------------------------
struct MyListener;

impl Listener for MyListener {
    fn notify(&mut self, event: &Event) {
        println!("Notify called with {:?} for MyListener", event);
        match *event {
            Event::Log(ref s) => {
                println!("Log {}", s);
            },
            _ => {}
        }
    }
}

struct MySecondListener;

impl Listener for MySecondListener {
    fn notify(&mut self, event: &Event) {
        println!("Notify called with {:?} for MySecondListener", event);
    }
}

fn events_register() {

    let mut d: Dispatcher = Dispatcher::new();
    let listener_rc = Arc::new(Mutex::new(MyListener));
    let second_listener_rc = Arc::new(Mutex::new(MySecondListener));
    d.register_listener(listener_rc.clone());
    d.register_listener(second_listener_rc.clone());
    d.dispatch(Event::Log("test".to_string()));
    d.dispatch(Event::Log("test".to_string()));
    d.dispatch(Event::Log("test".to_string()));
    d.dispatch(Event::Log("test".to_string()));
}

fn main() {
    events_register();
}
