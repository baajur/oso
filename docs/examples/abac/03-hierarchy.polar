# Hierarchies

## NEW CONCEPTS:
# - recursive attributes
# - representing hierachies

allow(actor: User, "view", resource: Expense) if
    employee = actor.employees() and
    employee.name = resource.submitted_by;

# start-manages-rule
allow(actor: User, "view", resource: Expense) if
    manages(actor, employee) and
    employee.name = resource.submitted_by;
# end-manages-rule

# start-hierarchy-rule
# Management hierarchies
manages(manager: User, employee) if
    employee = manager.employees()
    or manages(manager.employees(), employee);
# end-hierarchy-rule


# Now this inline query confirms Cora can view the expense because Cora manages
# Bhavik who manages Alice.
#?= allow(new User("cora"), "view", Expense.id(0));
