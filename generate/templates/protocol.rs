use std::marker::PhantomData;
use futures::{
    channel::mpsc::{UnboundedReceiver, UnboundedSender},
    executor, try_join,
};
#[allow(unused_imports)]
use ::rumpsteak::{
    channel::Bidirectional,
    session,
    Branch,
    End,
    Message,
    Receive,
    Role,
    Roles,
    Select,
    Send,
    effect::{
        SideEffect,
        Constant,
        Incr,
    },
    try_session,
    predicate::{
        Predicate,
        Tautology,
        LTnVar,
        GTnVar,
        LTnConst,
        GTnConst
    },

};

use std::collections::HashMap;
use std::error::Error;

type Channel = Bidirectional<UnboundedSender<Label>, UnboundedReceiver<Label>>;
type Name = {{ name_str }};
type Value = {{ value_str }};

pub struct AdHocPred<L, LHS: Predicate, RHS: Predicate> {
    _p: PhantomData<(L, LHS, RHS)>
}

impl<L, LHS: Predicate, RHS: Predicate> Default for AdHocPred<L, LHS, RHS> {
    fn default() -> Self { Self { _p: PhantomData} }
}

impl<L, LHS: Predicate, RHS: Predicate> Predicate for AdHocPred<L, LHS, RHS>
{
    type Name = name_str;
    type Value = value_str;
    type Label = L;
    type Error = ();

    fn check(&self, m: &HashMap<Self::Name, Self::Value>, l: Option<&Self::Label>) -> Result<(), Self::Error> {
        
        if let Some(l) = l {
            match l {
                l1 => LHS::default().check(m, None),
                l2 => RHS::default().check(m, None),
                _ => Err(())
            }
        } else {
            Ok(())
        }
    }
}
#[derive(Roles)]
#[allow(dead_code)]
struct Roles {
{%- for role in roles %}
    {{ role.snake }}: {{ role.camel }},
{%- endfor %}
}
{% for role in roles %}
#[derive(Role)]
#[message(Label)]
struct {{ role.camel }} {
{%- for index in role.routes.iter() %}
    {%- let route = roles[index.0] %}
    #[route({{ route.camel }})]
    {{ route.snake }}: Channel,
{%- endfor %}
}
{% endfor %}
#[derive(Message)]
enum Label {
{%- for label in labels %}
    {{ label.camel }}({{ label.camel }}),
{%- endfor %}
}
{% for label in labels %}
struct {{ label.camel }}{% if !label.parameters.is_empty() -%}
    ({{ label.parameters|join(", ") }})
{%- endif %};
{% endfor %}
{%- for role in roles %}
// role {{role.camel}}
{%- for (i, definition) in role.definitions.iter().rev().enumerate() %}
{%- let node = role.nodes[definition.node] %}
#[session(Name, Value)]
{%- match definition.body %}
{%- when DefinitionBody::Type with { safe, ty } %}
{%- if safe|copy_bool %}
type {{ camel }}{{ role.camel }}{% if i > 0 -%}{{ node }}{%- endif %} = {{ ty|ty(camel, role, roles, labels) }};
{%- else %}
struct {{ camel }}{{ role.camel }}{% if i > 0 -%}{{ node }}{%- endif %}({{ ty|ty(camel, role, roles, labels) }});
{%- endif %}
{%- when DefinitionBody::Choice with (choices) %}
enum {{ camel }}{{ role.camel }}{{ node }} {
{%- for choice in choices %}
    {%- let label = labels[choice.label] %}
    {{ label.camel }}({{ label.camel }}, {{ choice.ty|ty(camel, role, roles, labels) }}),
{%- endfor %}
}


{%- endmatch %}
{% endfor %}
{%- endfor %}
