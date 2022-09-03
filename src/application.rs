mod listener;
mod router;

use lunatic::supervisor::{Supervisor, SupervisorConfig, SupervisorStrategy};

use self::{router::Router, listener::Listener};

pub struct Application;

impl Application {    
}


impl Supervisor for Application {
    type Arg = ();

    type Children = (Router, Listener);

    fn init(config: &mut SupervisorConfig<Self>, _: ()) {
        config.set_strategy(SupervisorStrategy::OneForOne);
        config.children_args((((), Some("router".to_owned())), ((), Some("listener".to_owned()))));
    }
}