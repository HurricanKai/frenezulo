use std::collections::HashMap;

use lunatic::{Process, Mailbox};
use serde::{Serialize, Deserialize};

use crate::{module_supervisor::{ModuleSupervisorMessage, self}};
use frenezulo::{ ServiceId, RequestId, http::{Request, Response}};

type RespondTo = Process<Response>;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceRegistryMessage {
    StartRequest(RequestId, ServiceId, Request, RespondTo),
    CancelRequest(RequestId, ServiceId),
    CompleteRequest(RequestId, ServiceId, Response),
    AddService(ServiceId, serde_bytes::ByteBuf),
    DeleteService(ServiceId)
}

pub struct ServiceRegistry {
    services: HashMap<ServiceId, (Process<ModuleSupervisorMessage, crate::module_supervisor::WorkerSerializer>, HashMap<RequestId, (Request, RespondTo)>)>
}

impl ServiceRegistry {
    pub fn start_request(&mut self, service_id: ServiceId, request_id: RequestId, request: Request, respond_to: RespondTo) {
        match self.services.get_mut(&service_id) {
            Some((process, requests)) => {
                requests.insert(request_id, (request.clone(), respond_to));
                
                process.send(ModuleSupervisorMessage::StartRequest(request_id, request));
            },
            None => {
                println!("Invalid Service Id {service_id:?} Request: {request_id:?}");
                panic!("Invalid Service Id")
            }
        }
    }

    pub fn cancel_request(&mut self, service_id: ServiceId, request_id: RequestId) {
        match self.services.get_mut(&service_id) {
            Some((process, requests)) =>
                match requests.remove(&request_id) {
                    Some((request, response_process)) => {
                        process.send(ModuleSupervisorMessage::CancelRequest(request_id));
                        response_process.send(
                            submillisecond::response::Response::builder()
                                .status(503)
                                .version(request.metadata.version.into())
                                .body(b"Canceled Request".to_vec())
                                .expect("Request Builder must succeed")
                                .into()
                        );
                    },
                    None => ()
            },
            None => ()
        };

        todo!("Send cancel response");
    }

    pub fn add_service(&mut self, service_id: ServiceId, module_data: Vec<u8>) {
        let worker = module_supervisor::start(
            service_id.tag,
            service_id,
            module_data,
            Process::this());
        
        self.services.insert(service_id, (worker, HashMap::<RequestId, (Request, RespondTo)>::new()));
    }

    pub fn delete_service(&mut self, service_id: ServiceId) {
        match self.services.remove(&service_id) {
            Some((worker, mut table)) => {
                worker.kill();
                table.drain()
                    .for_each(|(_id, (request, process))| {
                        process.send(
                            submillisecond::response::Response::builder()
                                .status(404)
                                .version(request.metadata.version.into())
                                .body(b"Service Deleted".to_vec())
                                .expect("Request Builder must succeed")
                                .into());
                    });
            }
            None => (),
        }
    }

    pub fn complete_request(&mut self, request_id: RequestId, service_id: ServiceId, response: Response) {
        match self.services.get_mut(&service_id) {
            Some((_worker, table)) => {
                match table.remove(&request_id) {
                    Some((_request_id, respond_to)) => {
                        respond_to.send(response);
                    },
                    None => ()
                };
            },
            None => ()
        }
    }
}

pub fn start() -> Process<ServiceRegistryMessage> {
    Process::spawn_link((), |(), mailbox: Mailbox<ServiceRegistryMessage>| {
        println!("service registry started");
        mailbox.this().register("service_registry");
        println!("service registry registered");
        let mut instance = ServiceRegistry {
            services: HashMap::new()
        };

        let mailbox = mailbox.catch_link_failure();

        loop {
            let msg = mailbox.receive();
            match msg {
                lunatic::MailboxResult::Message(msg) => match msg {
                    ServiceRegistryMessage::StartRequest(request_id, service_id, request, respond_to) =>
                        instance.start_request(service_id, request_id, request, respond_to),
                    ServiceRegistryMessage::CancelRequest(request_id, service_id) =>
                        instance.cancel_request(service_id, request_id),
                    ServiceRegistryMessage::CompleteRequest(request_id, service_id, response) =>
                        instance.complete_request(request_id, service_id, response),
                    ServiceRegistryMessage::AddService(service_id, module_data) =>
                        instance.add_service(service_id, module_data.into_vec()),
                    ServiceRegistryMessage::DeleteService(service_id) =>
                        instance.delete_service(service_id),
                },
                lunatic::MailboxResult::DeserializationFailed(_) => todo!(),
                lunatic::MailboxResult::TimedOut => todo!(),
                lunatic::MailboxResult::LinkDied(_) => todo!("handle module supervisor crashes"), // Tag == service_id.tag
            }
        }
    })
}

pub fn start_request(request_id: RequestId, service_id: ServiceId, request: Request, respond_to: Process<Response>) {
    Process::<ServiceRegistryMessage>::lookup("service_registry")
        .expect("service registry has to be online")
        .send(ServiceRegistryMessage::StartRequest(request_id, service_id, request, respond_to))
}

pub fn cancel_request(request_id: RequestId, service_id: ServiceId) {
    Process::<ServiceRegistryMessage>::lookup("service_registry")
        .expect("service registry has to be online")
        .send(ServiceRegistryMessage::CancelRequest(request_id, service_id))
}

pub fn add_service(service_id: ServiceId, module_data: Vec<u8>) {
    Process::<ServiceRegistryMessage>::lookup("service_registry")
        .expect("service registry has to be online")
        .send(ServiceRegistryMessage::AddService(service_id, serde_bytes::ByteBuf::from(module_data)))
}

pub fn delete_service(service_id: ServiceId) {
    Process::<ServiceRegistryMessage>::lookup("service_registry")
        .expect("service registry has to be online")
        .send(ServiceRegistryMessage::DeleteService(service_id))
}