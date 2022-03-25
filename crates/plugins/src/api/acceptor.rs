use crate::{
    acceptor::{AcceptorPluginReceiver, AcceptorPluginSender},
    api::{PluginContext, PluginEnv},
};

pub fn has_send(env: &PluginEnv) -> i32 {
    let send_queue = match &env.context {
        PluginContext::Acceptor { send_queue_rx, .. } => send_queue_rx,
        _ => return -1,
    };

    let mut send_queue = send_queue
        .lock()
        .expect("Obtaining the Lock should never fail");

    let message = match send_queue.peek() {
        Some(m) => m,
        None => return -1,
    };

    let size = message.data.len() + 4;
    size as i32
}

pub fn send(env: &PluginEnv, target: i32) -> i32 {
    let send_queue = match &env.context {
        PluginContext::Acceptor { send_queue_rx, .. } => send_queue_rx,
        _ => return 0,
    };

    let mut send_queue = send_queue
        .lock()
        .expect("Obtaining the Lock should never fail");

    let message = match send_queue.try_recv() {
        Some(m) => m,
        None => return 0,
    };

    let size = message.data.len() + 4;
    let target = target as usize;

    let mem = env.get_memory_slice(target, size);
    mem.as_mut_slice()[..4].copy_from_slice(&message.id.to_be_bytes());
    mem.as_mut_slice()[4..].copy_from_slice(&message.data);

    1
}

pub fn new_con(env: &PluginEnv, id: i32) {
    let (start, send_queue_tx, connections) = match &env.context {
        PluginContext::Acceptor {
            start_handler,
            send_queue_tx,
            to_tl_queues,
            ..
        } => (start_handler.clone(), send_queue_tx, to_tl_queues),
        _ => return,
    };
    let send_queue = send_queue_tx
        .lock()
        .expect("Obtaining the Lock should never fail")
        .clone();

    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    let receiver = AcceptorPluginReceiver::new(rx);
    let sender = AcceptorPluginSender::new(id, send_queue);

    connections
        .lock()
        .expect("Obtaining the Lock should never fail")
        .insert(id, tx);

    tokio::task::spawn(start(id, receiver, sender));
}

pub fn recv(env: &PluginEnv, id: i32, addr: i32, size: i32) {
    let start = addr as usize;
    let size = size as usize;

    let mem = env.get_memory_slice(start, size);
    let data = mem.as_slice();

    let connections = match &env.context {
        PluginContext::Acceptor { to_tl_queues, .. } => to_tl_queues,
        _ => return,
    };

    let locked_cons = connections
        .lock()
        .expect("Obtaining the Lock should never fail");
    let connection = match locked_cons.get(&id) {
        Some(c) => c,
        None => return,
    };

    if let Err(e) = connection.send(data.to_vec()) {
        tracing::error!("Forwarding Plugin-Received-Data: {:?}", e);
    }
}
