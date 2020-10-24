use crate::counter::Counter;
use crate::error::PolarResult;
use crate::events::QueryEvent;
use crate::folder::Folder;
use crate::kb::Bindings;
use crate::partial::Constraints;
use crate::runnable::Runnable;
use crate::terms::{Operation, Operator, Term, Value};
use crate::vm::{Binding, BindingStack, Goals, PolarVirtualMachine};

#[derive(Clone)]
pub struct Inverter {
    vm: PolarVirtualMachine,
    results: Vec<Bindings>,
}

impl Inverter {
    pub fn new(vm: &PolarVirtualMachine, goals: Goals) -> Self {
        Self {
            vm: vm.clone_with_bindings(goals),
            results: vec![],
        }
    }
}

struct ConstraintInverter {
    pub new_bindings: BindingStack,
}

impl ConstraintInverter {
    pub fn new() -> Self {
        Self {
            new_bindings: vec![],
        }
    }
}

impl Folder for ConstraintInverter {
    fn fold_operation(&mut self, o: Operation) -> Operation {
        Operation {
            operator: match o.operator {
                Operator::And => Operator::Or,
                Operator::Or => Operator::And,
                Operator::Unify | Operator::Eq => Operator::Neq,
                Operator::Neq => Operator::Unify,
                Operator::Gt => Operator::Leq,
                Operator::Geq => Operator::Lt,
                Operator::Lt => Operator::Geq,
                Operator::Leq => Operator::Gt,
                _ => todo!("negate {:?}", o.operator),
            },
            args: self.fold_list(o.args),
        }
    }

    // If there are any constraints to invert, invert 'em.
    fn fold_constraints(&mut self, c: Constraints) -> Constraints {
        if !c.operations.is_empty() {
            let new_binding = Binding(
                c.variable.clone(),
                Term::new_temporary(Value::Partial(Constraints {
                    variable: c.variable.clone(),
                    operations: vec![Operation {
                        operator: Operator::Or,
                        args: c
                            .operations
                            .iter()
                            .cloned()
                            .map(|o| Term::new_temporary(Value::Expression(self.fold_operation(o))))
                            .collect(),
                    }],
                })),
            );
            self.new_bindings.push(new_binding);
        }
        c
    }
}

/// If there are no partials, and you get no results, return true
/// If there are no partials, and you get at least one result, return false
/// If there's a partial, return `true` with the partial.
///     - what if the partial has no operations?
impl Runnable for Inverter {
    fn run(
        &mut self,
        bindings: Option<&mut BindingStack>,
        bsp: Option<&mut usize>,
        _: Option<&mut Counter>,
    ) -> PolarResult<QueryEvent> {
        loop {
            // Pass most events through, but collect results and invert them.
            if let Ok(event) = self.vm.run(None, None, None) {
                match event {
                    QueryEvent::Done { .. } => {
                        if !self.results.is_empty() {
                            eprintln!("{:?}", self.results);
                            let inverted_results = self.results.into_iter().map(|result| {
                                let mut inverter = ConstraintInverter::new();
                                result.into_iter().for_each(|Binding(_, value)| {
                                    inverter.fold_term(value);
                                });
                            });

                            if let Some(parent_bindings) = bindings {
                                let bsp = bsp.expect("Inverter needs a BSP");
                                let mut inverter = ConstraintInverter::new();
                                self.vm
                                    .bindings
                                    .drain(*bsp..)
                                    .for_each(|Binding(_, value)| {
                                        inverter.fold_term(value);
                                    });
                                self.result = !inverter.new_bindings.is_empty();
                                *bsp += inverter.new_bindings.len();
                                parent_bindings.extend(inverter.new_bindings);
                            }
                        }
                        return Ok(QueryEvent::Done {
                            result: self.result,
                        });
                    }
                    QueryEvent::Result { bindings, .. } => {
                        self.results.push(bindings);
                    }
                    event => return Ok(event),
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
