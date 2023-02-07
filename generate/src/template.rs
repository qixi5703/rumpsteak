use super::Direction;
use askama::Template;
use std::{
    fmt::{self, Display, Formatter},
    fs::File,
    io::{self, Write},
    path::Path,
};

#[derive(Debug)]
pub(crate) struct Route(pub usize);

#[derive(Clone, Debug)]
pub(crate) enum Predicate {
    LTnVar(String, String),
    LTnConst(String, String),
    GTnVar(String, String),
    GTnConst(String, String),
    EqualVar(String, String),
    EqualConst(String, String),
}

impl Display for Predicate {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Predicate::LTnVar(param, value) => {
                write!(f, "LTnVar::<Value, Label, '{}', '{}'>", param, value)
            }
            Predicate::LTnConst(param, value) => {
                write!(f, "LTnConst::<Value, '{}', {}>", param, value)
            }
            Predicate::GTnVar(param, value) => {
                write!(f, "GTnVar::<Value, Label, '{}', '{}'>", param, value)
            }
            Predicate::GTnConst(param, value) => {
                write!(f, "GTnConst::<Value, '{}', {}>", param, value)
            }
            Predicate::EqualVar(param, value) => {
                write!(f, "EqualVar::<Value, Label, '{}', '{}'>", param, value)
            }
            Predicate::EqualConst(param, value) => {
                write!(f, "EqualConst::<Value, '{}', {}>", param, value)
            }
        }
    }
}

#[derive(Clone, Debug)]
pub(in crate) enum BoolPredicate {
    Normal(Predicate),
    And(Box<BoolPredicate>, Box<BoolPredicate>),
    Or(Box<BoolPredicate>, Box<BoolPredicate>),
    Neg(Box<BoolPredicate>),
    Tautology,
}

impl Display for BoolPredicate {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            BoolPredicate::Normal(a) => {
                write!(f, "{}", a)
            }
            BoolPredicate::Neg(a) => {
                write!(f, "Neg<{}>", a)
            }
            BoolPredicate::And(a, b) => {
                write!(f, "And<{}, {}>", a, b)
            }
            BoolPredicate::Or(a, b) => {
                write!(f, "Or<{}, {}>", a, b)
            }
            BoolPredicate::Tautology => {
                write!(f, "Tautology<Name, Value, Label>")
            }
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) enum SideEffect {
    Increase(String, String),
    Decrease(String, String),
    Multiply(String, String),
    Divide(String, String),
    None,
}

#[derive(Debug)]
pub(crate) enum Type {
    End,
    Node(usize),
    Message {
        direction: Direction,
        role: usize,
        label: usize,
        predicate: BoolPredicate,
        side_effect: SideEffect,
        next: Box<Self>,
    },
    Choice {
        direction: Direction,
        role: usize,
        node: usize,
        predicate: BoolPredicate,
        side_effect: SideEffect,
    },
}

impl Type {
    pub(crate) fn is_choice(&self) -> bool {
        matches!(
            self,
            Self::Choice {
                direction: _,
                role: _,
                node: _,
                predicate: _,
                side_effect: _,
            }
        )
    }
}

struct TypeFormatter<'a> {
    ty: &'a Type,
    name: &'a str,
    role: &'a Role,
    roles: &'a [Role],
    labels: &'a [Label],
}

impl<'a> TypeFormatter<'a> {
    fn with(&self, ty: &'a Type) -> Self {
        Self {
            ty,
            name: self.name,
            role: self.role,
            roles: self.roles,
            labels: self.labels,
        }
    }

    fn role(&self, role: &usize) -> &str {
        &self.roles[*role].camel
    }

    fn label(&self, label: &usize) -> &str {
        &self.labels[*label].camel
    }

    fn node(&self, node: &usize) -> &str {
        &self.role.nodes[*node]
    }

    fn pred(&self, predicate: &Predicate) -> String {
        predicate.to_string()
    }

    fn boolpred(&self, predicate: &BoolPredicate) -> String {
        predicate.to_string()
    }

    fn effect(&self, side_effect: &SideEffect) -> String {
        match side_effect {
            SideEffect::Increase(param, value) => {
                let mut effect = String::from("Incr<'");
                effect = effect + param;
                effect = effect + "', ";
                effect = effect + value;
                effect = effect + ">";
                return effect;
            }
            SideEffect::Decrease(param, value) => {
                let mut effect = String::from("Decr<'");
                effect = effect + param;
                effect = effect + "', ";
                effect = effect + value;
                effect = effect + ">";
                return effect;
            }
            SideEffect::Multiply(param, value) => {
                let mut effect = String::from("Mult<'");
                effect = effect + param;
                effect = effect + "', ";
                effect = effect + value;
                effect = effect + ">";
                return effect;
            }
            SideEffect::Divide(param, value) => {
                let mut effect = String::from("Div<'");
                effect = effect + param;
                effect = effect + "', ";
                effect = effect + value;
                effect = effect + ">";
                return effect;
            }
            SideEffect::None => (),
        }
        return "Constant<Name, Value>".to_string();
    }
}

impl Display for TypeFormatter<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.ty {
            Type::End => write!(f, "End"),
            Type::Node(node) if *node > 0 => {
                write!(f, "{}{}{}", self.name, self.role.camel, self.node(node))
            }
            Type::Node(_) => write!(f, "{}{}", self.name, self.role.camel),
            Type::Message {
                direction,
                role,
                label,
                predicate,
                side_effect,
                next,
            } => {
                let (other, label, pred, effect, next) = (
                    self.role(role),
                    self.label(label),
                    self.boolpred(predicate),
                    self.effect(side_effect),
                    self.with(next),
                );
                match direction {
                    Direction::Send => write!(
                        f,
                        "Send<{}, {}, {}, {}, {}>",
                        other, label, pred, effect, next
                    ),
                    Direction::Receive => write!(
                        f,
                        "Receive<{}, {}, {}, {}, {}>",
                        other, label, pred, effect, next
                    ),
                }
            }
            Type::Choice {
                direction,
                role,
                node,
                predicate,
                side_effect,
            } => {
                let other = self.role(role);
                let (other, name, role, node, pred, effect) = (
                    self.role(role),
                    self.name,
                    &self.role.camel,
                    self.node(node),
                    self.boolpred(predicate),
                    self.effect(side_effect),
                );
                match direction {
                    Direction::Send => {
                        write!(
                            f,
                            "Select<{}, {}{}{}Predicate, {}, {}{}{}>",
                            other, name, role, node, effect, name, role, node
                        )
                    }
                    Direction::Receive => {
                        write!(
                            f,
                            "Branch<{}, {}, {}, {}{}{}>",
                            other, pred, effect, name, role, node
                        )
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
pub(crate) struct Choice {
    pub label: usize,
    pub ty: Type,
    pub predicate: BoolPredicate,
}

#[derive(Debug)]
pub(crate) enum DefinitionBody {
    Type { safe: bool, ty: Type },
    Choice(Vec<Choice>),
}

#[derive(Debug)]
pub(crate) struct Definition {
    pub node: usize,
    pub body: DefinitionBody,
}

#[derive(Debug)]
pub(crate) struct Role {
    pub camel: String,
    pub snake: String,
    pub nodes: Vec<String>,
    pub routes: Vec<Route>,
    pub definitions: Vec<Definition>,
}

#[derive(Debug)]
pub(crate) struct Label {
    pub camel: String,
    pub param_names: Vec<String>,
    pub parameters: Vec<String>,
}

#[derive(Debug, Template)]
#[template(path = "protocol.rs", escape = "none")]
pub struct Protocol {
    pub(crate) camel: String,
    pub(crate) roles: Vec<Role>,
    pub(crate) labels: Vec<Label>,
}

impl Protocol {
    pub fn write_to_file(&self, path: impl AsRef<Path>) -> io::Result<()> {
        write!(File::create(path)?, "{}", self)
    }
}

mod filters {
    use super::{Label, Role, Type, TypeFormatter};
    use askama::Result;

    #[allow(clippy::unnecessary_wraps)]
    pub(super) fn copy_bool(b: &bool) -> Result<bool> {
        Ok(*b)
    }

    #[allow(clippy::unnecessary_wraps)]
    pub(super) fn ty<'a>(
        ty: &'a Type,
        name: &'a str,
        role: &'a Role,
        roles: &'a [Role],
        labels: &'a [Label],
    ) -> Result<TypeFormatter<'a>> {
        Ok(TypeFormatter {
            ty,
            name,
            role,
            roles,
            labels,
        })
    }
}
