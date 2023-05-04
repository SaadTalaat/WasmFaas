use crate::streams::{duplex, Stream};
use std::any::Any;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::sync::Mutex;
use std::thread::{self, JoinHandle};
use std::time::Duration;
//use tokio::sync::mpsc::{channel, Receiver, Sender};
use crossbeam_channel::{
    unbounded as channel, Receiver, RecvError, SendError, Sender, TryRecvError,
};

pub struct Registry {
    handle: JoinHandle<()>,
    stream: Stream<FEProtocol, FEProtocol>,
    nodes: Mutex<RefCell<Vec<WorkerHandle>>>,
    counter: Mutex<Cell<u32>>,
    rr: Mutex<Cell<u32>>,
}

impl Registry {
    pub fn start() -> Self {
        let (stream_1, stream_2) = duplex();
        let handle = thread::spawn(|| {
            let mut backend = RegistryBackend::new(stream_1);
            backend.run();
        });
        Self {
            handle,
            stream: stream_2,
            nodes: Mutex::new(RefCell::new(vec![])),
            counter: Mutex::new(Cell::new(0)),
            rr: Mutex::new(Cell::new(0)),
        }
    }

    pub fn register(&self) -> BackendHandle {
        let id: u32 = self.get_worker_id();
        let (channel_1, channel_2) = duplex();
        let info = WorkerHandle::new(id, channel_1);
        self.nodes.lock().unwrap().borrow_mut().push(info);
        //self.send(FEProtocol::Register(info));
        BackendHandle::new(id, channel_2)
    }

    fn get_worker_id(&self) -> u32 {
        let id = self.counter.lock().unwrap().get();
        self.counter.lock().unwrap().set(id + 1);
        id
    }

    pub fn disconnect(&self, stream: &BackendHandle) {
        self.send(FEProtocol::Disconnect(stream.id))
    }

    pub fn invoke(&self, fn_name: String) -> String {
        self.send(FEProtocol::Invoke(fn_name));
        if let FEProtocol::Result(r) = self.recv() {
            r
        } else {
            panic!("Invalid result")
        }
    }
    pub fn send(&self, msg: FEProtocol) {
        self.stream.send(msg).unwrap();
    }
    pub fn recv(&self) -> FEProtocol {
        self.stream.recv().unwrap()
    }

    pub fn join(self) -> Result<(), Box<dyn Any + Send>> {
        self.handle.join()
    }
}

struct RegistryBackend {
    stream: Stream<FEProtocol, FEProtocol>,
    lookup: HashMap<u32, usize>,
    nodes: Vec<WorkerHandle>,
}

impl RegistryBackend {
    pub fn new(stream: Stream<FEProtocol, FEProtocol>) -> Self {
        Self {
            stream,
            lookup: HashMap::new(),
            nodes: vec![],
        }
    }

    pub fn run(&mut self) {
        let mut counter = 0;
        println!("Running");
        loop {
            if let Ok(msg) = self.stream.try_recv() {
                match msg {
                    FEProtocol::Die => {
                        println!("Dying..");
                        break;
                    }
                    // TODO: use oneshot channels instead of maintaining
                    // channels for workers
                    FEProtocol::Invoke(name) => {
                        let worker_idx = counter;
                        counter = (counter + 1) % self.nodes.len();
                        let worker = self.nodes.get(worker_idx).unwrap();
                        println!("Invoking {} on worker {}", name, worker_idx);
                        let result = worker.invoke(name).unwrap();
                        println!("Backend received result: {:?}", result);
                        if let WorkerProtocol::InvokeResult(r) = result {
                            self.stream.send(FEProtocol::Result(r)).unwrap();
                        } else {
                            panic!("Reply from worker is not InvokeResult: {:?}", result)
                        }
                    }
                    FEProtocol::Register(info) => {
                        println!("Registering worker {:?}", info.id);
                        let idx = self.nodes.len();
                        self.lookup.insert(info.id, idx);
                        self.nodes.push(info)
                    }
                    FEProtocol::Disconnect(id) => {
                        println!("Disconnected {}", id);
                        let idx = self.lookup.remove(&id).unwrap();
                        self.nodes.remove(idx);
                    }
                    _ => panic!("Illegal message: {:?}", msg),
                }
            } else {
                thread::sleep(Duration::from_millis(10))
            }
        }
    }
}

/// Single stream between backend -> worker
#[derive(Debug)]
pub struct WorkerHandle {
    id: u32,
    stream: Stream<BEProtocol, WorkerProtocol>,
}

impl WorkerHandle {
    pub fn new(id: u32, stream: Stream<BEProtocol, WorkerProtocol>) -> Self {
        Self { id, stream }
    }

    pub fn invoke(&self, fn_name: String) -> Result<WorkerProtocol, RecvError> {
        println!("Sending WorkerHandle::");
        self.stream.send(BEProtocol::Invoke(fn_name)).unwrap();
        println!("Receiving WorkerHandle::");
        let res = self.stream.recv();
        println!("Received WorkerHandle::");
        res
    }
}

/// single stream between worker -> backend
#[derive(Debug)]
pub struct BackendHandle {
    pub id: u32,
    stream: Stream<WorkerProtocol, BEProtocol>,
}

impl BackendHandle {
    pub fn new(id: u32, stream: Stream<WorkerProtocol, BEProtocol>) -> Self {
        Self { id, stream }
    }

    pub fn send(&self, msg: WorkerProtocol) -> Result<(), SendError<WorkerProtocol>> {
        self.stream.send(msg)
    }

    pub fn recv(&self) -> Result<BEProtocol, RecvError> {
        self.stream.recv()
    }

    pub fn try_recv(&self) -> Result<BEProtocol, TryRecvError> {
        self.stream.try_recv()
    }
}

#[derive(Debug)]
pub enum FEProtocol {
    Register(WorkerHandle),
    Disconnect(u32),
    Invoke(String),
    Result(String),
    Die,
}

#[derive(Debug)]
pub enum BEProtocol {
    Invoke(String),
    WorkerDie,
}

#[derive(Debug)]
pub enum WorkerProtocol {
    InvokeResult(String),
    Dead,
}
