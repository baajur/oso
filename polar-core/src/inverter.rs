use crate::counter::Counter;
use crate::error::PolarResult;
use crate::events::QueryEvent;
use crate::kb::Bindings;
use crate::runnable::Runnable;
use crate::terms::Term;
use crate::vm::{BindingStack, Goals, PolarVirtualMachine};

#[derive(Clone)]
pub struct Inverter {
    vm: PolarVirtualMachine,
    results: Vec<Bindings>, // FIXME: no traces.
}

impl Inverter {
    pub fn new(vm: &PolarVirtualMachine, goals: Goals) -> Self {
        Self {
            vm: vm.clone_with_bindings(goals),
            results: vec![],
        }
    }
}

impl Runnable for Inverter {
    fn run(
        &mut self,
        bindings: Option<&mut BindingStack>,
        mut counter: &mut Counter,
    ) -> PolarResult<QueryEvent> {
        eprintln!("[Inverter] BEFORE RUN: {:?}\n", bindings);

        loop {
            // Pass most events through, but collect results and invert them.
            if let Ok(event) = self.vm.run(None, &mut counter) {
                match event {
                    QueryEvent::Done { .. } => {
                        eprintln!("[Inverter] DONE - RESULTS: {:?}\n", self.results);
                        eprintln!("[Inverter] DONE - returning: {}\n", self.results.is_empty());

                        let result = !self.results.is_empty();

                        eprintln!("[Inverter] AFTER RUN: {:?}\n", bindings);

                        let x = if let Some(parent_bindings) = bindings {
                            // let inverter_bindings = self.results.drain(..).flat_map(|bindings| {
                            //     bindings
                            //         .into_iter()
                            //         .map(|(k, v)| Binding(k, v))
                            //         .collect::<BindingStack>()
                            // });
                            // parent_bindings.extend(inverter_bindings);

                            parent_bindings.clear();
                            parent_bindings.extend(self.vm.bindings.drain(..));
                            parent_bindings.clone()
                        } else {
                            vec![]
                        };

                        eprintln!("[Inverter] AFTER MOVE: {:?}\n", x);

                        return Ok(QueryEvent::Done { result });
                    }
                    QueryEvent::Result { bindings, .. } => {
                        eprintln!("[Inverter] RESULT - BINDINGS: {:?}\n", bindings);
                        self.results.push(bindings);
                    }
                    event => {
                        eprintln!("[Inverter] {:?}\n", event);
                        return Ok(event);
                    }
                }
            }
        }
    }

    fn external_question_result(&mut self, call_id: u64, answer: bool) -> PolarResult<()> {
        self.vm.external_question_result(call_id, answer)
    }

    fn external_error(&mut self, message: String) -> PolarResult<()> {
        self.vm.external_error(message)
    }

    fn external_call_result(&mut self, call_id: u64, term: Option<Term>) -> PolarResult<()> {
        self.vm.external_call_result(call_id, term)
    }

    fn clone_runnable(&self) -> Box<dyn Runnable> {
        Box::new(self.clone())
    }
}
