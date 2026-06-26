use super::*;

impl TypeChecker {
    pub(crate) fn is_number_type(&self, ty: &Type) -> bool {
        matches!(ty, Type::Number | Type::NumberLiteral(_))
    }

    pub(crate) fn is_string_type(&self, ty: &Type) -> bool {
        matches!(ty, Type::String | Type::StringLiteral(_))
    }

    pub(crate) fn _is_boolean_type(&self, ty: &Type) -> bool {
        matches!(ty, Type::Boolean | Type::BooleanLiteral(_))
    }

    pub(crate) fn is_compatible(&self, expected: &Type, actual: &Type) -> bool {
        if expected == actual {
            return true;
        }
        if matches!(expected, Type::Any) || matches!(actual, Type::Any) {
            return true;
        }
        if matches!(expected, Type::Unknown) || matches!(actual, Type::Unknown) {
            return true;
        }
        if matches!(expected, Type::Never) {
            return true;
        }
        match (expected, actual) {
            (Type::Union(types), _) => types.iter().any(|t| self.is_compatible(t, actual)),
            (_, Type::Union(types)) => types.iter().any(|t| self.is_compatible(expected, t)),
            (Type::Number, Type::NumberLiteral(_)) => true,
            (Type::NumberLiteral(_), Type::Number) => true,
            (Type::String, Type::StringLiteral(_)) => true,
            (Type::StringLiteral(_), Type::String) => true,
            (Type::Boolean, Type::BooleanLiteral(_)) => true,
            (Type::BooleanLiteral(_), Type::Boolean) => true,
            (Type::Array(inner_exp), Type::Array(inner_act)) => {
                self.is_compatible(inner_exp, inner_act)
            }
            (Type::Tuple(t1), Type::Tuple(t2)) => {
                t1.len() == t2.len()
                    && t1.iter()
                        .zip(t2.iter())
                        .all(|(a, b)| self.is_compatible(a, b))
            }
            (Type::Tuple(t1), Type::Array(inner)) => {
                t1.iter().all(|t| self.is_compatible(t, inner))
            }
            (Type::Array(inner), Type::Tuple(t2)) => {
                t2.iter().all(|t| self.is_compatible(inner, t))
            }
            (Type::Object(props_exp), Type::Object(props_act)) => {
                for prop_exp in props_exp {
                    let found = props_act.iter().find(|p| p.name == prop_exp.name);
                    match found {
                        Some(prop_act) => {
                            if !prop_exp.optional && !self.is_compatible(&prop_exp.ty, &prop_act.ty) {
                                return false;
                            }
                        }
                        None => {
                            if !prop_exp.optional {
                                return false;
                            }
                        }
                    }
                }
                true
            }
            (Type::Named(name), _) => {
                if let Some(iface) = self.interfaces.get(name) {
                    self.is_compatible(&Type::Object(iface.properties.clone()), actual)
                } else if let Some(alias) = self.type_aliases.get(name) {
                    self.is_compatible(&alias.ty, actual)
                } else {
                    false
                }
            }
            (_, Type::Named(name)) => {
                if let Some(iface) = self.interfaces.get(name) {
                    self.is_compatible(expected, &Type::Object(iface.properties.clone()))
                } else if let Some(alias) = self.type_aliases.get(name) {
                    self.is_compatible(expected, &alias.ty)
                } else {
                    false
                }
            }
            (Type::Function { params: p1, return_type: r1 }, Type::Function { params: p2, return_type: r2 }) => {
                p1.len() == p2.len()
                    && p1.iter().zip(p2.iter()).all(|(a, b)| self.is_compatible(a, b))
                    && self.is_compatible(r1, r2)
            }
            _ => false,
        }
    }
}
