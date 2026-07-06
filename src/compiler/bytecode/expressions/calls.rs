use super::*;

impl CodeGenerator {
    pub(super) fn generate_call(
        &mut self,
        callee: &Expression,
        args: &[Expression],
    ) -> Result<()> {
        if let Expression::Member {
            object,
            property,
            computed,
        } = callee
        {
            if !computed {
                if let Expression::Identifier(name) = property.as_ref() {
                    match name.as_str() {
                        "set" if args.len() == 2 => {
                            self.generate_expression(object)?;
                            for arg in args {
                                self.generate_expression(arg)?;
                            }
                            self.emit(Instruction::MapSet(args.len() as u16));
                            return Ok(());
                        }
                        "get" if args.len() == 1 => {
                            self.generate_expression(object)?;
                            for arg in args {
                                self.generate_expression(arg)?;
                            }
                            self.emit(Instruction::MapGet);
                            return Ok(());
                        }
                        "has" if args.len() == 1 => {
                            self.generate_expression(object)?;
                            for arg in args {
                                self.generate_expression(arg)?;
                            }
                            self.emit(Instruction::MapHas);
                            return Ok(());
                        }
                        "delete" if args.len() == 1 => {
                            self.generate_expression(object)?;
                            for arg in args {
                                self.generate_expression(arg)?;
                            }
                            self.emit(Instruction::MapDelete);
                            return Ok(());
                        }
                        "add" if args.len() == 1 => {
                            self.generate_expression(object)?;
                            for arg in args {
                                self.generate_expression(arg)?;
                            }
                            self.emit(Instruction::SetAdd);
                            return Ok(());
                        }
                        _ => {}
                    }
                }
            }
            self.generate_expression(object)?;
            if *computed {
                self.generate_expression(property)?;
            } else if let Expression::Identifier(name) = property.as_ref() {
                let idx = self.add_constant(Value::String(name.clone()));
                self.emit(Instruction::LoadConst(idx));
            } else {
                self.generate_expression(property)?;
            }
            for arg in args {
                self.generate_expression(arg)?;
            }
            self.emit(Instruction::CallMethod(args.len() as u16));
        } else if let Expression::OptionalMember {
            object,
            property,
            computed,
        } = callee
        {
            self.generate_expression(object)?;
            self.emit(Instruction::Dup);
            let check_undef = self.instructions.len();
            self.emit(Instruction::JumpIfUndefined(0));
            if *computed {
                self.generate_expression(property)?;
            } else if let Expression::Identifier(name) = property.as_ref() {
                let idx = self.add_constant(Value::String(name.clone()));
                self.emit(Instruction::LoadConst(idx));
            } else {
                self.generate_expression(property)?;
            }
            for arg in args {
                self.generate_expression(arg)?;
            }
            self.emit(Instruction::CallMethod(args.len() as u16));
            let skip_end = self.instructions.len();
            self.emit(Instruction::Jump(0));
            self.patch_jump(check_undef, self.instructions.len());
            self.emit(Instruction::Pop);
            self.emit(Instruction::LoadUndefined);
            self.patch_jump(skip_end, self.instructions.len());
        } else {
            for arg in args {
                self.generate_expression(arg)?;
            }
            self.generate_expression(callee)?;
            self.emit(Instruction::Call(args.len() as u16));
        }
        Ok(())
    }

    pub(super) fn generate_optional_call(
        &mut self,
        callee: &Expression,
        args: &[Expression],
    ) -> Result<()> {
        if let Expression::Member {
            object,
            property,
            computed,
        } = callee
        {
            self.generate_expression(object)?;
            self.emit(Instruction::Dup);
            let check_undef = self.instructions.len();
            self.emit(Instruction::JumpIfUndefined(0));
            if *computed {
                self.generate_expression(property)?;
            } else if let Expression::Identifier(name) = property.as_ref() {
                let idx = self.add_constant(Value::String(name.clone()));
                self.emit(Instruction::LoadConst(idx));
            } else {
                self.generate_expression(property)?;
            }
            for arg in args {
                self.generate_expression(arg)?;
            }
            self.emit(Instruction::CallMethod(args.len() as u16));
            let skip_end = self.instructions.len();
            self.emit(Instruction::Jump(0));
            self.patch_jump(check_undef, self.instructions.len());
            self.emit(Instruction::Pop);
            self.emit(Instruction::LoadUndefined);
            self.patch_jump(skip_end, self.instructions.len());
        } else if let Expression::OptionalMember {
            object,
            property,
            computed,
        } = callee
        {
            self.generate_expression(object)?;
            self.emit(Instruction::Dup);
            let check_undef = self.instructions.len();
            self.emit(Instruction::JumpIfUndefined(0));
            if *computed {
                self.generate_expression(property)?;
            } else if let Expression::Identifier(name) = property.as_ref() {
                let idx = self.add_constant(Value::String(name.clone()));
                self.emit(Instruction::LoadConst(idx));
            } else {
                self.generate_expression(property)?;
            }
            for arg in args {
                self.generate_expression(arg)?;
            }
            self.emit(Instruction::CallMethod(args.len() as u16));
            let skip_end = self.instructions.len();
            self.emit(Instruction::Jump(0));
            self.patch_jump(check_undef, self.instructions.len());
            self.emit(Instruction::Pop);
            self.emit(Instruction::LoadUndefined);
            self.patch_jump(skip_end, self.instructions.len());
        } else {
            self.generate_expression(callee)?;
            self.emit(Instruction::Dup);
            let check_undef = self.instructions.len();
            self.emit(Instruction::JumpIfUndefined(0));
            for arg in args {
                self.generate_expression(arg)?;
            }
            self.emit(Instruction::Call(args.len() as u16));
            let skip_end = self.instructions.len();
            self.emit(Instruction::Jump(0));
            self.patch_jump(check_undef, self.instructions.len());
            self.emit(Instruction::Pop);
            self.emit(Instruction::LoadUndefined);
            self.patch_jump(skip_end, self.instructions.len());
        }
        Ok(())
    }

    pub(super) fn generate_new_expression(
        &mut self,
        callee: &Expression,
        args: &[Expression],
    ) -> Result<()> {
        self.generate_expression(callee)?;
        for arg in args {
            self.generate_expression(arg)?;
        }
        self.emit(Instruction::Construct(args.len() as u16));
        Ok(())
    }
}
