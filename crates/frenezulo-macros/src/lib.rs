extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_attribute]
pub fn handler(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input: syn::ItemFn = match syn::parse(item.clone()) {
        Ok(it) => it,
        Err(e) => return token_stream_with_error(item, e),
    };

    if input.sig.inputs.len() != 1 {
        let msg = "must be on a function with 1 argument of type Request";
        return syn::Error::new_spanned(&input.sig.ident, msg)
            .to_compile_error()
            .into();
    }

    let arguments = input.sig.inputs;
    let block = input.block;
    let result = input.sig.output;

    quote! {
        #[export_name = "frenezulo_main"]
        extern "C" fn frenezulo_main() {
            run(unsafe { lunatic::Mailbox::<frenezulo::WorkerMessage, frenezulo::WorkerSerializer>::new() })
        }

        #[lunatic::main]
        fn main(mailbox: lunatic::Mailbox::<frenezulo::WorkerMessage, frenezulo::WorkerSerializer>) {
            run(mailbox)
        }

        fn run(mailbox: lunatic::Mailbox<frenezulo::WorkerMessage, frenezulo::WorkerSerializer>) {
            match mailbox.receive() {
                //MailboxResult::Message(msg) => match msg {
                    frenezulo::WorkerMessage::Request(request_id, request, respond_to) => {
                        let response = __handle(request);
                        respond_to.send(frenezulo::ModuleSupervisorMessage::CompleteRequest(
                            request_id, response,
                        ));
                    }
                //},
            }
        }

        fn __handle(#arguments) #result {
            #block
        }
    }
    .into()
}

fn token_stream_with_error(mut tokens: TokenStream, error: syn::Error) -> TokenStream {
    tokens.extend(TokenStream::from(error.into_compile_error()));
    tokens
}
