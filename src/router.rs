use std::collections::HashMap;

use lunatic::{process::ProcessRef, Tag, abstract_process};

use crate::{service_registry};
use frenezulo::{ ServiceId, RequestId};

pub struct Router(HashMap<String, ServiceId>);

#[abstract_process]
impl Router {
    #[init]
    fn init(_: ProcessRef<Self>, _: ()) -> Self {
        Self(HashMap::new())
    } 

    #[terminate]
    fn terminate(self) {
    }

    #[handle_link_trapped]
    fn handle_link_trapped(&self, _tag: Tag) {
        println!("Link trapped");
    }

    #[handle_request]
    fn add_service(&mut self, prefix: String, data: serde_bytes::ByteBuf) -> ServiceId {
        let id = ServiceId { tag: Tag::new() };
        self.0.insert(prefix.clone(), id);
        service_registry::add_service(id, data.into_vec());
        println!("Registered service {prefix:?} {id:?}");
        id
    }

    #[handle_request]
    fn create_request(&self, prefix: String) -> Option<(ServiceId, RequestId)> {
        let service_id = self.0.get(&prefix)?;
        let request_id = RequestId {
            tag: Tag::new()
        };
        Some((*service_id, request_id))
    }
}

pub fn create_request(prefix: String) -> Option<(ServiceId, RequestId)> {
    ProcessRef::<Router>::lookup("router").expect("router has to be found").create_request(prefix)
}

pub fn add_service(prefix: String, module_data: Vec<u8>) -> ServiceId {
    ProcessRef::<Router>::lookup("router").expect("router has to be found")
        .add_service(prefix, serde_bytes::ByteBuf::from(module_data))
}