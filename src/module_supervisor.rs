use std::{collections::HashMap, time::Duration};

use frenezulo::{WorkerMessage, http::{Version, ResponseMetadata}};
use lunatic::{WasmModule, Process, LunaticError, ProcessConfig, Tag, Mailbox};
use multimap::MultiMap;
use serde::{Serialize, Deserialize};

use crate::{service_registry::ServiceRegistryMessage};
use frenezulo::{ ServiceId, RequestId, http::{Request, Response}};

pub struct ModuleSupervisor {
    service_id: ServiceId,
    module: WasmModule,
    supervisor: Process<ServiceRegistryMessage>,
    outstanding_requests: HashMap<RequestId, Process<WorkerMessage, WorkerSerializer>>,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModuleSupervisorMessage {
    StartRequest(RequestId, Request),
    CancelRequest(RequestId),
    CompleteRequest(RequestId, Response)
}

impl ModuleSupervisor {
    fn respond(&self, request_id: RequestId, response: Response) {
        self.supervisor.send(ServiceRegistryMessage::CompleteRequest(request_id, self.service_id, response));
    }

    pub fn start_request(&mut self, request_id: RequestId, request: Request) {
        static MODULE_TIMEOUT : Duration = Duration::from_millis(30);
        let timeout_response : Response = Response {
            metadata: ResponseMetadata {
                headers: MultiMap::new(),
                status: 504,
                version: Version::Http11
            },
            body: "Service timed out".to_owned().into_bytes()
        };

        let mut config = ProcessConfig::new().expect("needs to be able to create configs");
        config.set_max_memory(1024 * 1024 * 1024);

        let new_worker : Result<Process<WorkerMessage, WorkerSerializer>, LunaticError> = self.module.spawn_link_config_tag::<WorkerMessage, WorkerSerializer>(
            "random_bullshit_go", Some(&config), Some(request_id.tag), &[]);
        match new_worker {
            Ok(worker) => {
                self.outstanding_requests.insert(request_id, worker.clone());
                worker.send(WorkerMessage::Request(request_id, request, Process::this()));
                self.supervisor.send_after(ServiceRegistryMessage::CompleteRequest(request_id, self.service_id, timeout_response), MODULE_TIMEOUT);
            },
            Err(err) => {
                println!("Failed to start worker {err:?}");
                let response = submillisecond::http::Response::builder()
                    .status(503)
                    .version(request.metadata.version.into())
                    .body(b"503 - Failed to start service".to_vec()).expect("Build 503 has to be possible");
                self.respond(request_id, response.into());
            }
        }
    }

    pub fn cancel_request(&mut self, request_id: RequestId) {
        match self.outstanding_requests.remove(&request_id) {
            Some(worker) => {
                worker.kill();
            }
            None => ()
        }
    }

    pub fn complete_request(&mut self, request_id: RequestId, response: Response) {
        match self.outstanding_requests.remove(&request_id) {
            Some(worker) => {
                self.respond(request_id, response);
                worker.kill();
            }
            None => ()
        }
    }
}

pub type WorkerSerializer = frenezulo::module_supervisor::WorkerSerializer;
pub fn start(tag: Tag, service_id: ServiceId, module_data: Vec<u8>, supervisor: Process<ServiceRegistryMessage>) -> Process<ModuleSupervisorMessage, WorkerSerializer> {
    println!("starting module supervisor");
    let mut config = ProcessConfig::new().expect("needs to be able to create configs");
    config.set_can_spawn_processes(true);
    config.set_can_create_configs(true);
    config.set_can_compile_modules(true);

    println!("spawning module supervisor");
    Process::spawn_link_config_tag(&config, (service_id, module_data, supervisor), tag,
    |(service_id, module_data, supervisor), mailbox: Mailbox<ModuleSupervisorMessage, WorkerSerializer>| 
    {
        let me = mailbox.this();
        let mailbox = mailbox.catch_link_failure();
        println!("compiling module {me:?}");
        let module = WasmModule::new(&module_data);
        if let Err(e) = module {
            println!("Failed to compile {e:?}");
            panic!("Failed to compile {e:?}");
        }
        println!("done compiling");
        
        let mut instance = ModuleSupervisor {
            service_id,
            supervisor,
            module: module.unwrap(),
            outstanding_requests: HashMap::new()
        };

        loop {
            match mailbox.try_receive(Duration::MAX) {
                lunatic::MailboxResult::Message(msg) =>
                    match msg {
                        ModuleSupervisorMessage::StartRequest(request_id, request) =>
                            instance.start_request(request_id, request),
                        ModuleSupervisorMessage::CancelRequest(request_id) =>
                            instance.cancel_request(request_id),
                        ModuleSupervisorMessage::CompleteRequest(request_id, response) =>
                            instance.complete_request(request_id, response),
                    },
                lunatic::MailboxResult::DeserializationFailed(err) => {println!("Deserialization Failed {err:?}"); panic!("Deserialization Failed {err:?}");},
                lunatic::MailboxResult::TimedOut => todo!(),
                lunatic::MailboxResult::LinkDied(tag) => {
                    let request_id = RequestId { tag };
                    match instance.outstanding_requests.remove(&request_id) {
                        Some(_worker) => {
                            instance.supervisor.send(ServiceRegistryMessage::CancelRequest(request_id, service_id));
                        },
                        None => ()
                    }
                },
            }
        }
    })
}