use super::*;

impl CodeGenerator {
    pub(super) fn generate_number_literal(&mut self, n: f64) -> Result<()> {
        let idx = self.add_constant(Value::Float(n));
        self.emit(Instruction::LoadConst(idx));
        Ok(())
    }

    pub(super) fn generate_bigint_literal(&mut self, s: &str) -> Result<()> {
        let val: i128 = s.parse().unwrap_or(0);
        let idx = self.add_constant(Value::BigInt(val));
        self.emit(Instruction::LoadConst(idx));
        Ok(())
    }

    pub(super) fn generate_string_literal(&mut self, s: &str) -> Result<()> {
        let idx = self.add_constant(Value::from_string(s.to_string()));
        self.emit(Instruction::LoadConst(idx));
        Ok(())
    }

    pub(super) fn generate_regex_literal(&mut self, pattern: &str, flags: &str) -> Result<()> {
        self.emit(Instruction::LoadGlobal("RegExp".to_string()));
        let reg_idx = self.add_constant(Value::from_string(pattern.to_string()));
        self.emit(Instruction::LoadConst(reg_idx));
        if !flags.is_empty() {
            let flags_idx = self.add_constant(Value::from_string(flags.to_string()));
            self.emit(Instruction::LoadConst(flags_idx));
            self.emit(Instruction::Construct(2));
        } else {
            self.emit(Instruction::Construct(1));
        }
        Ok(())
    }

    pub(super) fn generate_boolean_literal(&mut self, b: bool) -> Result<()> {
        if b {
            self.emit(Instruction::LoadTrue);
        } else {
            self.emit(Instruction::LoadFalse);
        }
        Ok(())
    }

    pub(super) fn generate_nan_literal(&mut self) -> Result<()> {
        let idx = self.add_constant(Value::Float(f64::NAN));
        self.emit(Instruction::LoadConst(idx));
        Ok(())
    }

    pub(super) fn generate_infinity_literal(&mut self) -> Result<()> {
        let idx = self.add_constant(Value::Float(f64::INFINITY));
        self.emit(Instruction::LoadConst(idx));
        Ok(())
    }

    pub(super) fn generate_template_literal(
        &mut self,
        quasis: &[String],
        expressions: &[Expression],
    ) -> Result<()> {
        if expressions.is_empty() {
            let s = quasis.join("");
            let idx = self.add_constant(Value::from_string(s.into()));
            self.emit(Instruction::LoadConst(idx));
        } else {
            let first = &quasis[0];
            if !first.is_empty() {
                let idx = self.add_constant(Value::from_string(first.clone().into()));
                self.emit(Instruction::LoadConst(idx));
            }

            for i in 0..expressions.len() {
                if first.is_empty() && i == 0 {
                    self.generate_expression(&expressions[i])?;
                    self.emit(Instruction::ToString);
                } else {
                    self.generate_expression(&expressions[i])?;
                    self.emit(Instruction::ToString);
                    self.emit(Instruction::Add);
                }

                if i + 1 < quasis.len() && !quasis[i + 1].is_empty() {
                    let idx = self.add_constant(Value::from_string(quasis[i + 1].clone().into()));
                    self.emit(Instruction::LoadConst(idx));
                    self.emit(Instruction::Add);
                }
            }
        }
        Ok(())
    }
}
