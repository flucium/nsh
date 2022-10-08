use crate::variable::Variable;

pub fn set(variable: &mut Variable, key: String, val: String) {
    variable.insert(key, val)
}

pub fn unset(variable: &mut Variable, key: String) {
    variable.remove(key)
}