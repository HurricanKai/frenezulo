use std::{collections::HashMap, fs::File, io::Read};

use lunatic::{abstract_process, process::ProcessRef, Tag, WasmModule};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Router(HashMap<String, u64>);

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
        panic!("Router trapped link, but no links should exist")
    }

    #[handle_message]
    fn add_service(&mut self, prefix: String, url: String) {
        // TODO: Move this to HTTP instead
        let mut file = File::open(url).expect("failed to open WASM file");
        let metadata = file.metadata().expect("failed to read metadata");
        let mut buffer = Vec::with_capacity(metadata.len() as usize);
        file.read(&mut buffer).expect("failed to read file");

        let result = WasmModule::new(&buffer);

        if let Ok(WasmModule::Module(i)) = result {
            self.0.insert(prefix.to_owned(), i);
        }
        else if let Err(err) = result {
            println!("Failed to compile: {}", err);
        }
    }

    #[handle_request]
    fn get_service(&self, prefix: String) -> Option<u64> {
        self.0.get(&prefix).map(|x| x.to_owned())
    }
}

fn get_process() -> ProcessRef<Router> {
    ProcessRef::lookup("router").expect("failed to find router")
}

pub fn add_service(prefix: String, url: String) {
    get_process().add_service(prefix, url)
}

pub fn get_service(prefix: String) -> Option<WasmModule> {
    get_process().get_service(prefix)
    .map(|i| WasmModule::Module(i))
} 