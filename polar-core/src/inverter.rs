use crate::counter::Counter;
use crate::error::PolarResult;
use crate::events::QueryEvent;
use crate::runnable::Runnable;
use crate::terms::Term;
use crate::vm::{Goals, PolarVirtualMachine};

#[derive(Clone)]
pub struct Inverter {
    vm: PolarVirtualMachine,
}

impl Inverter {
    pub fn new(vm: &PolarVirtualMachine, goals: Goals) -> Self {
        Self {
            vm: vm.clone_with_bindings(goals),
        }
    }
}

impl Runnable for Inverter {
    fn run(&mut self, counter: Counter) -> PolarResult<QueryEvent> {
        self.vm.run(counter).map(|r| match r {
            QueryEvent::Done { result } => QueryEvent::Done { result: !result },
            event => event,
        })
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
