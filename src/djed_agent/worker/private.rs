use crate::callback::Callback;
use std::fmt;
use std::marker::PhantomData;
use web_sys::{Worker};
use crate::djed_agent::{
    agent::{Agent, HandlerId, Discoverer, Bridge},
    worker::{send_to_remote, FromWorker, ToWorker, worker_new}
};
use serde::{Deserialize, Serialize};


const SINGLETON_ID: HandlerId = HandlerId(0, true);

/// Create a new instance for every bridge.
#[allow(missing_debug_implementations)]
pub struct Private<AGN> {
    _agent: PhantomData<AGN>,
}

impl<AGN> Discoverer for Private<AGN>
where
    AGN: Agent,
    <AGN as Agent>::Input: Serialize + for<'de> Deserialize<'de>,
    <AGN as Agent>::Output: Serialize + for<'de> Deserialize<'de>,
{
    type Agent = AGN;

    fn spawn_or_join(callback: Option<Callback<AGN::Output>>) -> Box<dyn Bridge<AGN>> {
        let callback = callback.expect("Callback required for Private agents");
        let handler = move |data: Vec<u8>, worker: &Worker| {
            let msg = FromWorker::<AGN::Output>::unpack(&data);
            match msg {
                FromWorker::WorkerLoaded => {
                    send_to_remote::<AGN>(&worker, ToWorker::Connected(SINGLETON_ID));
                }
                FromWorker::ProcessOutput(id, output) => {
                    assert_eq!(id.raw_id(), SINGLETON_ID.raw_id());
                    callback.emit(output);
                }
            }
        };

        // TODO(#947): Drop handler when bridge is dropped
        let name_of_resource = AGN::name_of_resource();
        let worker = {
                let worker = worker_new(name_of_resource, AGN::is_module());
                let worker_clone = worker.clone();
                worker.set_onmessage_closure(move |data: Vec<u8>| handler(data, &worker_clone));
                worker
        };
        let bridge = PrivateBridge {
            worker,
            _agent: PhantomData,
        };
        Box::new(bridge)
    }
}

/// A connection manager for components interaction with workers.
pub struct PrivateBridge<AGN>
where
    AGN: Agent,
    <AGN as Agent>::Input: Serialize + for<'de> Deserialize<'de>,
    <AGN as Agent>::Output: Serialize + for<'de> Deserialize<'de>,
{
    worker: Worker,
    _agent: PhantomData<AGN>,
}

impl<AGN> fmt::Debug for PrivateBridge<AGN>
where
    AGN: Agent,
    <AGN as Agent>::Input: Serialize + for<'de> Deserialize<'de>,
    <AGN as Agent>::Output: Serialize + for<'de> Deserialize<'de>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PrivateBridge<_>")
    }
}

impl<AGN> Bridge<AGN> for PrivateBridge<AGN>
where
    AGN: Agent,
    <AGN as Agent>::Input: Serialize + for<'de> Deserialize<'de>,
    <AGN as Agent>::Output: Serialize + for<'de> Deserialize<'de>,
{
    fn send(&mut self, msg: AGN::Input) {
        // TODO(#937): Important! Implement.
        // Use a queue to collect a messages if an instance is not ready
        // and send them to an agent when it will reported readiness.
        let msg = ToWorker::ProcessInput(SINGLETON_ID, msg).pack();
        self.worker.post_message_vec(msg)
    }
}

impl<AGN> Drop for PrivateBridge<AGN>
where
    AGN: Agent,
    <AGN as Agent>::Input: Serialize + for<'de> Deserialize<'de>,
    <AGN as Agent>::Output: Serialize + for<'de> Deserialize<'de>,
{
    fn drop(&mut self) {
        let disconnected = ToWorker::Disconnected(SINGLETON_ID);
        send_to_remote::<AGN>(&self.worker, disconnected);

        let destroy = ToWorker::Destroy;
        send_to_remote::<AGN>(&self.worker, destroy);
    }
}