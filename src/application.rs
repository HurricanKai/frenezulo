
use lunatic::{supervisor::{Supervisor, SupervisorConfig, SupervisorStrategy}, Process, abstract_process, process::ProcessRef, Tag};

use crate::service_registry::{ServiceRegistryMessage, self};

use crate::{router::Router, listener::Listener};

pub struct Application;

impl Application {    
}

pub struct ServiceRegistryWrapper(Process<ServiceRegistryMessage>);

#[abstract_process]
impl ServiceRegistryWrapper {
    #[init]
    fn init(_: ProcessRef<Self>, _: ()) -> Self {
        let process = service_registry::start();
        Self(process)
    }

    #[terminate]
    fn terminate(self) {
        self.0.kill();
    }

    #[handle_link_trapped]
    fn handle_link_trapped(&self, _tag: Tag) {
        panic!("Underlying Process failed");
    }
}

impl Supervisor for Application {
    type Arg = ();

    type Children = (ServiceRegistryWrapper, Router, Listener);

    fn init(config: &mut SupervisorConfig<Self>, _: ()) {
        config.set_strategy(SupervisorStrategy::OneForOne);
        config.children_args((
            ((), None),
            ((), Some("router".to_owned())),
            ((), Some("listener".to_owned()))
        ));
    }
}