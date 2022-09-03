use core::panic;
use std::{collections::HashMap, time::Duration, string::ParseError};

use generic_error::{GenErr, GenericError};
use lunatic::{Mailbox, Process, WasmModule, LunaticError, ProcessConfig};
use multimap::MultiMap;
use serde::{Serialize, Deserialize};

use crate::http::{Request, Response};

use super::{RouterMessage, RequestId, ServiceId};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
enum SupervisorMessage {
    StartRequest(RequestId, ServiceId, Request),
    CancelRequest(RequestId, ServiceId),
    CompleteRequest(RequestId, ServiceId, Response),
    AddService(ServiceId, u64),
    DeleteService(ServiceId)
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
struct SupervisorInfo {
    supervisor: Process<RouterMessage>
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
struct RequestInfo {
    id: RequestId,
    request: Request
}

pub fn supervise_modules(info: SupervisorInfo, mailbox: Mailbox<SupervisorMessage>) {
    let this = mailbox.this();
    let child_config = ProcessConfig::new();
    child_config.set_can_spawn_processes(true);
    child_config.set_max_memory(1024 * 1024 * 1024); /* 1mbi */

    let mailbox = mailbox.catch_link_failure();
    let mut services = HashMap::<ServiceId, Process<ModuleSupervisorMessage>>::new();
    let mut outstanding_requests = MultiMap::<ServiceId, RequestInfo>::new();

    loop {
        match mailbox.receive() {
            lunatic::MailboxResult::Message(msg) => match msg {
                SupervisorMessage::StartRequest(id, service_id, request) => {
                    match services.get(&service_id) {
                        Some(process) => {
                            outstanding_requests.insert(service_id, RequestInfo { id, request });
                            process.send(ModuleSupervisorMessage::StartRequest(id, request));
                        },
                        None => {
                            println!("Invalid Service Id {service_id:?} Request: {id:?}");
                            todo!("Respond with 404")
                        }
                    }
                },
                SupervisorMessage::CancelRequest(id, service_id) => {
                    match services.get(&service_id) {
                        Some(process) => {
                            match outstanding_requests.get_vec_mut(&service_id) {
                                Some(mut vec) => {
                                    match vec.iter().position(|e| e.id == id) {
                                        Some(i) => {
                                            process.send(ModuleSupervisorMessage::CancelRequest(id));
                                            vec.remove(i);
                                            Ok(())
                                        }
                                        None => GenErr!("Failed to find Id in outstanding requests")
                                    }
                                },
                                None => GenErr!("Failed to retrieve mutable vector for service id")
                            }
                        },
                        None => GenErr!("Failed to get service")
                    }.unwrap();
                    todo!("Send cancel response");
                },
                SupervisorMessage::CompleteRequest(id, service_id, Response) => {
                    todo!("Send Response");
                },
                SupervisorMessage::AddService(service_id, wasm_module_id) => {
                    let worker = Process::spawn_link_config_tag(&child_config,
                        ModuleSupervisorInfo {
                            service_id,
                            wasm_module_id,
                            supervisor: this.clone(),
                        },
                        service_id.tag,
                        supervise_single_module);
                    
                    services.insert(service_id, worker);
                },
                SupervisorMessage::DeleteService(service_id) => {
                    match services.remove(&service_id) {
                        Some(worker) => {
                            worker.kill();
                        }
                        None => (),
                    }
                }
            },
            lunatic::MailboxResult::LinkDied(_tag) => {
                todo!("handle crashes");
            },
            lunatic::MailboxResult::TimedOut => {
                panic!("Not using timeout")
            },
            lunatic::MailboxResult::DeserializationFailed(error) => {
                panic!("Deserialization should only fail across module boundaries?! {error:?}")
            }
        }
    }
}


#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
enum ModuleSupervisorMessage {
    StartRequest(RequestId, Request),
    CancelRequest(RequestId),
    CompleteRequest(RequestId, Response)
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
struct ModuleSupervisorInfo {
    service_id: ServiceId,
    wasm_module_id: u64,
    supervisor: Process<SupervisorMessage>
}

const MODULE_TIMEOUT : Duration = Duration::from_millis(30);

fn supervise_single_module(info: ModuleSupervisorInfo, mailbox: Mailbox<ModuleSupervisorMessage>) {
    let module = WasmModule::Module(info.wasm_module_id);
    let this = mailbox.this();
    let mailbox = mailbox.catch_link_failure();
    let mut requests = HashMap::<RequestId, Process<WorkerMessage>>::new();

    loop {
        match mailbox.try_receive(Duration::from_secs(5)) {
            lunatic::MailboxResult::Message(msg) => match msg {
                ModuleSupervisorMessage::StartRequest(id, request) => {
                    let new_worker : Result<Process<WorkerMessage>, LunaticError> = module.spawn_link("main", &[]);
                    match new_worker {
                        Ok(worker) => {
                            requests.insert(id, worker.clone());
                            worker.send(WorkerMessage::Request(id, request, this.clone()));
                            info.supervisor.send_after(SupervisorMessage::CancelRequest(id, info.service_id,), MODULE_TIMEOUT);
                        },
                        Err(err) => {
                            println!("Failed to start worker {err:?}");
                            let response = submillisecond::http::Response::builder()
                                .status(503)
                                .version(request.metadata.version.into())
                                .body(b"503 - Failed to start service".to_vec()).expect("Build 503 has to be possible");
                            info.supervisor.send(SupervisorMessage::CompleteRequest(id, info.service_id, response.into()));
                        }
                    }
                }
                ModuleSupervisorMessage::CancelRequest(id) => {
                    match requests.remove(&id) {
                        Some(worker) => {
                            worker.kill();
                        }
                        None => ()
                    }
                }
                ModuleSupervisorMessage::CompleteRequest(id, response) => {
                    info.supervisor.send(SupervisorMessage::CompleteRequest(id, info.service_id, response));
                    match requests.remove(&id) {
                        Some(worker) => {
                            worker.kill();
                        }
                        None => ()
                    }
                }
            },
            lunatic::MailboxResult::DeserializationFailed(error) => {
                println!("Error during module deserialization. Restarting all module workers. This may lead to dropping requests. Error: {error:?}");
                panic!("Deserialization Failed {error:?}");
            },
            lunatic::MailboxResult::LinkDied(_tagg) => {
                todo!("handle crashes");
            },
            lunatic::MailboxResult::TimedOut => {
                // don't care
            },
        }
    }
}



#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
enum WorkerMessage {
    Request(RequestId, Request, Process<ModuleSupervisorMessage>)
}